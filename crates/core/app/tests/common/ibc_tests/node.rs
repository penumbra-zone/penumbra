use {
    crate::common::{BuilderExt as _, TempStorageExt as _},
    anyhow::{anyhow, Context as _, Result},
    cnidarium::TempStorage,
    ed25519_consensus::{SigningKey, VerificationKey},
    ibc_proto::ibc::core::{
        channel::v1::query_client::QueryClient as IbcChannelQueryClient,
        client::v1::query_client::QueryClient as IbcClientQueryClient,
        connection::v1::query_client::QueryClient as IbcConnectionQueryClient,
    },
    ibc_types::{
        core::{
            channel::{ChannelEnd, ChannelId, PortId, Version as ChannelVersion},
            client::{ClientId, ClientType, Height},
            connection::{
                ChainId, ConnectionEnd, ConnectionId, Counterparty, Version as ConnectionVersion,
            },
        },
        lightclients::tendermint::{
            consensus_state::ConsensusState, header::Header as TendermintHeader,
        },
    },
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_ibc::{component::ClientStateReadExt as _, IBC_COMMITMENT_PREFIX},
    penumbra_sdk_keys::test_keys,
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::util::tendermint_proxy::v1::{
        tendermint_proxy_service_client::TendermintProxyServiceClient, GetStatusRequest,
    },
    std::error::Error,
    tap::{Tap, TapFallible},
    tendermint::{
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        vote::Power,
        Time,
    },
    tokio::time,
    tonic::transport::Channel,
    tower_actor::Actor,
    tracing::info,
};

// Contains some data from a single IBC connection + client for test usage.
// This might be better off as an extension trait or additional impl on the TestNode struct.
#[allow(unused)]
pub struct TestNodeWithIBC {
    pub connection_id: ConnectionId,
    pub channel_id: ChannelId,
    pub client_id: ClientId,
    pub port_id: PortId,
    pub chain_id: String,
    pub counterparty: Counterparty,
    pub connection_version: ConnectionVersion,
    pub channel_version: ChannelVersion,
    pub signer: String,
    pub connection: Option<ConnectionEnd>,
    pub channel: Option<ChannelEnd>,
    pub node: TestNode<Actor<ConsensusRequest, ConsensusResponse, Box<dyn Error + Send + Sync>>>,
    pub storage: TempStorage,
    pub ibc_client_query_client: IbcClientQueryClient<Channel>,
    pub ibc_connection_query_client: IbcConnectionQueryClient<Channel>,
    pub ibc_channel_query_client: IbcChannelQueryClient<Channel>,
    pub tendermint_proxy_service_client: TendermintProxyServiceClient<Channel>,
}

#[allow(unused)]
/// This interacts with a node similarly to how a relayer would. We intentionally call
/// against the external gRPC interfaces to get the most comprehensive test coverage.
impl TestNodeWithIBC {
    pub async fn new(
        suffix: &str,
        start_time: Time,
        keys: (SigningKey, VerificationKey),
    ) -> Result<Self, anyhow::Error> {
        let chain_id = format!("{}-{}", TestNode::<()>::CHAIN_ID, suffix);
        // Use the correct substores
        let storage = TempStorage::new_with_penumbra_prefixes().await?;
        // Instantiate a mock tendermint proxy, which we will connect to the test node.
        let proxy = penumbra_sdk_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();

        let node = {
            let app_state =
                AppState::Content(genesis::Content::default().with_chain_id(chain_id.clone()));
            let consensus = Consensus::new(storage.as_ref().clone());
            TestNode::builder()
                .with_keys(vec![keys])
                .single_validator()
                .with_initial_timestamp(start_time)
                .with_penumbra_auto_app_state(app_state)?
                .on_block(proxy.on_block_callback())
                .init_chain(consensus)
                .await
                .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
        };

        // to select a port number just index on the suffix for now
        let index = match suffix {
            "a" => 0,
            "b" => 1,
            _ => unreachable!("update this hack"),
        };
        // We use a non-standard port range, to avoid conflicting with other
        // integration tests that bind to the more typical 8080/8081 ports.
        let grpc_url = format!("http://127.0.0.1:999{}", index) // see #4517
            .parse::<url::Url>()?
            .tap(|url| tracing::debug!(%url, "parsed grpc url"));

        tracing::info!("spawning gRPC...");
        // Spawn the node's RPC server.
        let _rpc_server = {
            let make_svc = penumbra_sdk_app::rpc::routes(
                storage.as_ref(),
                proxy,
                false, /*enable_expensive_rpc*/
            )?
            .into_axum_router()
            .layer(tower_http::cors::CorsLayer::permissive())
            .into_make_service()
            .tap(|_| tracing::info!("initialized rpc service"));
            let [addr] = grpc_url
                .socket_addrs(|| None)?
                .try_into()
                .expect("grpc url can be turned into a socket address");

            let server = axum_server::bind(addr).serve(make_svc);
            tokio::spawn(async { server.await.expect("grpc server returned an error") })
                .tap(|_| tracing::info!("grpc server is running"))
        };

        time::sleep(time::Duration::from_secs(1)).await;
        // Create an RPC server for each chain to respond to IBC-related queries.
        let channel = Channel::from_shared(grpc_url.to_string())
            .with_context(|| "could not parse node URI")?
            .connect()
            .await
            .with_context(|| "could not connect to grpc server")
            .tap_err(|error| tracing::error!(?error, "could not connect to grpc server"))?;

        let ibc_connection_query_client = IbcConnectionQueryClient::new(channel.clone());
        let ibc_channel_query_client = IbcChannelQueryClient::new(channel.clone());
        let ibc_client_query_client = IbcClientQueryClient::new(channel.clone());
        let tendermint_proxy_service_client = TendermintProxyServiceClient::new(channel.clone());

        let pk = node
            .keyring()
            .iter()
            .next()
            .expect("validator key in keyring")
            .0;
        let proposer_address = tendermint::account::Id::new(
            <sha2::Sha256 as sha2::Digest>::digest(pk).as_slice()[0..20]
                .try_into()
                .expect(""),
        );
        Ok(Self {
            // the test relayer supports only a single connection on each chain as of now
            connection_id: ConnectionId::new(0),
            // the test relayer supports only a single channel per connection on each chain as of now
            channel_id: ChannelId::new(0),
            // Only ICS20 transfers are supported
            port_id: PortId::transfer(),
            node,
            storage,
            client_id: ClientId::new(ClientType::new("07-tendermint".to_string()), 0)?,
            chain_id: chain_id.clone(),
            counterparty: Counterparty {
                client_id: ClientId::new(ClientType::new("07-tendermint".to_string()), 0)?,
                connection_id: None,
                prefix: IBC_COMMITMENT_PREFIX.to_owned(),
            },
            connection_version: ConnectionVersion::default(),
            channel_version: ChannelVersion::new("ics20-1".to_string()),
            signer: hex::encode_upper(proposer_address),
            connection: None,
            channel: None,
            ibc_connection_query_client,
            ibc_channel_query_client,
            ibc_client_query_client,
            tendermint_proxy_service_client,
        })
    }

    pub async fn client(&mut self) -> Result<MockClient, anyhow::Error> {
        // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
        Ok(MockClient::new(test_keys::SPEND_KEY.clone())
            .with_sync_to_storage(&self.storage)
            .await?
            .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage")))
    }

    pub async fn get_latest_height(&mut self) -> Result<Height, anyhow::Error> {
        let status: penumbra_sdk_proto::util::tendermint_proxy::v1::GetStatusResponse = self
            .tendermint_proxy_service_client
            .get_status(GetStatusRequest {})
            .await?
            .into_inner();
        Ok(Height::new(
            ChainId::chain_version(&self.chain_id),
            status
                .sync_info
                .ok_or(anyhow!("no sync info"))?
                .latest_block_height,
        )?)
    }

    // TODO: maybe move to an IBC extension trait for TestNode?
    // or maybe the Block has everything it needs to produce this?
    pub fn create_tendermint_header(
        &self,
        trusted_height: Option<Height>,
        penumbra_sdk_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse {
            block_id: _,
            block,
        }: penumbra_sdk_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse,
    ) -> Result<TendermintHeader> {
        let pk = self
            .node
            .keyring()
            .iter()
            .next()
            .expect("validator key in keyring")
            .0;
        let block = block.ok_or(anyhow!("no block"))?;
        let header = block.header.ok_or(anyhow!("no header"))?;

        // the tendermint SignedHeader is non_exhaustive so we
        // can't use struct syntax to instantiate it and have to do
        // some annoying manual construction of the pb type instead.
        let h: tendermint::block::Header = header.clone().try_into().expect("bad header");
        use tendermint_proto::v0_37::types::SignedHeader as RawSignedHeader;
        // The SignedHeader is the header accompanied by the commit to prove it.
        let rsh: RawSignedHeader = RawSignedHeader {
            header: Some(tendermint_proto::v0_37::types::Header {
                version: Some(tendermint_proto::v0_37::version::Consensus {
                    block: header.version.as_ref().expect("version").block,
                    app: header.version.expect("version").app,
                }),
                chain_id: header.chain_id,
                height: header.height.into(),
                time: Some(tendermint_proto::google::protobuf::Timestamp {
                    seconds: header.time.as_ref().expect("time").seconds,
                    nanos: header.time.expect("time").nanos,
                }),
                last_block_id: header.last_block_id.clone().map(|a| {
                    tendermint_proto::v0_37::types::BlockId {
                        hash: a.hash,
                        part_set_header: a.part_set_header.map(|b| {
                            tendermint_proto::v0_37::types::PartSetHeader {
                                total: b.total,
                                hash: b.hash,
                            }
                        }),
                    }
                }),
                last_commit_hash: header.last_commit_hash.into(),
                data_hash: header.data_hash.into(),
                validators_hash: header.validators_hash.into(),
                next_validators_hash: header.next_validators_hash.into(),
                consensus_hash: header.consensus_hash.into(),
                app_hash: header.app_hash.into(),
                last_results_hash: header.last_results_hash.into(),
                evidence_hash: header.evidence_hash.into(),
                proposer_address: header.proposer_address.into(),
            }),
            commit: Some(tendermint_proto::v0_37::types::Commit {
                // The commit is for the current height
                height: header.height.into(),
                round: 0.into(),
                block_id: Some(tendermint_proto::v0_37::types::BlockId {
                    hash: h.hash().into(),
                    part_set_header: Some(tendermint_proto::v0_37::types::PartSetHeader {
                        total: 0,
                        hash: vec![],
                    }),
                }),
                // signatures for this block
                signatures: self
                    .node
                    .last_commit()
                    .unwrap()
                    .signatures
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
            }),
        };

        let signed_header = rsh.clone().try_into()?;

        // now get a SignedHeader
        let pub_key =
            tendermint::PublicKey::from_raw_ed25519(pk.as_bytes()).expect("pub key present");
        let proposer_address = tendermint::account::Id::new(
            <sha2::Sha256 as sha2::Digest>::digest(pk).as_slice()[0..20]
                .try_into()
                .expect(""),
        );
        // TODO: don't hardcode these
        let validator_set = tendermint::validator::Set::new(
            vec![tendermint::validator::Info {
                address: proposer_address.try_into()?,
                pub_key,
                power: Power::try_from(25_000 * 10i64.pow(6))?,
                name: Some("test validator".to_string()),
                proposer_priority: 1i64.try_into()?,
            }],
            // Same validator as proposer?
            Some(tendermint::validator::Info {
                address: proposer_address.try_into()?,
                pub_key,
                power: Power::try_from(25_000 * 10i64.pow(6))?,
                name: Some("test validator".to_string()),
                proposer_priority: 1i64.try_into()?,
            }),
        );

        // now we can make the Header
        let header = TendermintHeader {
            signed_header,
            validator_set: validator_set.clone(),
            trusted_validator_set: validator_set.clone(),
            trusted_height: trusted_height.unwrap_or_else(|| ibc_types::core::client::Height {
                revision_number: 0,
                revision_height: 0,
            }),
        };
        Ok(header)
    }
}
