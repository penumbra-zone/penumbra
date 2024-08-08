use {
    self::common::BuilderExt,
    anyhow::{anyhow, Context as _},
    cnidarium::{Snapshot, TempStorage},
    ibc_proto::ibc::{
        core::{
            channel::v1::{
                query_client::QueryClient as IbcConsensusQueryClient,
                QueryChannelConsensusStateRequest,
            },
            client::v1::{
                query_client::QueryClient as IbcClientQueryClient, QueryClientStateRequest,
                QueryConsensusStateRequest,
            },
            connection::v1::{
                query_client::QueryClient as IbcConnectionQueryClient, QueryConnectionRequest,
            },
        },
        lightclients::tendermint::v1::ClientState,
    },
    ibc_types::{
        core::{
            channel::{ChannelId, PortId},
            client::{
                msgs::{MsgCreateClient, MsgUpdateClient},
                ClientId, ClientType, Height,
            },
            commitment::{MerklePrefix, MerkleProof, MerkleRoot},
            connection::{
                events::ConnectionOpenInit,
                msgs::{MsgConnectionOpenInit, MsgConnectionOpenTry},
                ConnectionEnd, ConnectionId, Counterparty, State as ConnectionState, Version,
            },
        },
        lightclients::tendermint::{client_state::AllowUpdate, TrustThreshold},
        path::ClientStatePath,
        DomainType as _,
    },
    penumbra_app::{
        genesis::{self, AppState},
        server::consensus::{self, Consensus},
    },
    penumbra_asset::asset,
    penumbra_community_pool::{CommunityPoolDeposit, StateReadExt},
    penumbra_ibc::{
        component::{
            ChannelStateReadExt as _, ClientStateReadExt as _, ConnectionStateReadExt as _,
        },
        IbcRelay, MerklePrefixExt as _, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS,
    },
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_num::Amount,
    penumbra_proto::DomainType,
    penumbra_shielded_pool::SpendPlan,
    penumbra_transaction::{TransactionParameters, TransactionPlan},
    prost::Message as _,
    rand_core::OsRng,
    std::{
        collections::BTreeMap,
        error::Error,
        io::Read as _,
        str::FromStr as _,
        time::{Duration, SystemTime},
    },
    tap::{Tap, TapFallible},
    tendermint::{
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        Hash, Time,
    },
    tokio::time,
    tonic::transport::Channel,
    tower_actor::Actor,
    tracing::info,
};

mod common;

// Contains some data from a single IBC connection + client for test usage.
// TODO: this elides some behavior that is exercised when a real relayer
// for example Hermes relays between Penumbra chains. Specifically,
// in these tests we are keeping an internal cache of data that is normally fetched from the
// chain. This may hide potential bugs in the RPC interface used by relayers.
// The implementation here should be changed to fetch data from the chain instead.
struct TestIbcData {
    connection_id: ConnectionId,
    client_id: ClientId,
    chain_id: String,
    counterparty: Counterparty,
    version: Version,
    signer: String,
    connection: Option<ConnectionEnd>,
    node: TestNode<Actor<ConsensusRequest, ConsensusResponse, Box<dyn Error + Send + Sync>>>,
    client: MockClient,
    storage: TempStorage,
    ibc_client_query_client: IbcClientQueryClient<Channel>,
    ibc_connection_query_client: IbcConnectionQueryClient<Channel>,
    ibc_consensus_query_client: IbcConsensusQueryClient<Channel>,
}

impl TestIbcData {
    async fn new(suffix: &str) -> Result<Self, anyhow::Error> {
        let chain_id = format!("{}-{}", TestNode::<()>::CHAIN_ID, suffix);
        let storage = TempStorage::new().await?;
        // Instantiate a mock tendermint proxy, which we will connect to the test node.
        let proxy = penumbra_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();

        let mut node = {
            let app_state =
                AppState::Content(genesis::Content::default().with_chain_id(chain_id.clone()));
            let consensus = Consensus::new(storage.as_ref().clone());
            TestNode::builder()
                .single_validator()
                .with_penumbra_auto_app_state(app_state)?
                .on_block(proxy.on_block_callback())
                .init_chain(consensus)
                .await
                .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
        };

        // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
        let client = MockClient::new(test_keys::SPEND_KEY.clone())
            .with_sync_to_storage(&storage)
            .await?
            .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

        // TODO: hacky lol
        let (other_suffix, index) = match suffix {
            "a" => ("b", 0),
            "b" => ("a", 1),
            _ => unreachable!("update this hack"),
        };
        let grpc_url = format!("http://127.0.0.1:808{}", index) // see #4517
            .parse::<url::Url>()?
            .tap(|url| tracing::debug!(%url, "parsed grpc url"));

        println!("spawning gRPC...");
        // Spawn the node's RPC server.
        let _rpc_server = {
            let make_svc = penumbra_app::rpc::router(
                storage.as_ref(),
                proxy,
                false, /*enable_expensive_rpc*/
            )?
            .into_router()
            .layer(tower_http::cors::CorsLayer::permissive())
            .into_make_service()
            .tap(|_| println!("initialized rpc service"));
            let [addr] = grpc_url
                .socket_addrs(|| None)?
                .try_into()
                .expect("grpc url can be turned into a socket address");
            let server = axum_server::bind(addr).serve(make_svc);
            tokio::spawn(async { server.await.expect("grpc server returned an error") })
                .tap(|_| println!("grpc server is running"))
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
        let ibc_consensus_query_client = IbcConsensusQueryClient::new(channel.clone());
        let ibc_client_query_client = IbcClientQueryClient::new(channel.clone());

        Ok(Self {
            // the test relayer supports only a single connection on each chain as of now
            connection_id: ConnectionId::new(0),
            node,
            client,
            storage,
            client_id: ClientId::new(ClientType::new("07-tendermint".to_string()), 0)?,
            chain_id: chain_id.clone(),
            counterparty: Counterparty {
                client_id: ClientId::new(ClientType::new("07-tendermint".to_string()), 0)?,
                connection_id: None,
                prefix: MerklePrefix::try_from(
                    format!("chain {}", other_suffix).as_bytes().to_vec(),
                )?,
            },
            version: Version::default(),
            signer: format!("chain {} signer", suffix).to_string(),
            connection: None,
            ibc_connection_query_client,
            ibc_consensus_query_client,
            ibc_client_query_client,
        })
    }
}

struct TestRelayer {
    chain_a_ibc: TestIbcData,
    chain_b_ibc: TestIbcData,
}

impl TestRelayer {
    async fn merkle_root_a(&self) -> Result<MerkleRoot, anyhow::Error> {
        let final_snapshot_a = self.chain_a_ibc.storage.latest_snapshot();
        let final_root_a = final_snapshot_a.root_hash().await?;
        Ok(MerkleRoot {
            hash: final_root_a.0.to_vec(),
        })
    }

    async fn merkle_root_b(&self) -> Result<MerkleRoot, anyhow::Error> {
        let final_snapshot_b = self.chain_b_ibc.storage.latest_snapshot();
        let final_root_b = final_snapshot_b.root_hash().await?;
        Ok(MerkleRoot {
            hash: final_root_b.0.to_vec(),
        })
    }

    async fn _handshake(&mut self) -> Result<(), anyhow::Error> {
        // The IBC connection handshake has four steps (Init, Try, Ack, Confirm).
        // https://github.com/cosmos/ibc/blob/main/spec/core/ics-003-connection-semantics/README.md
        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L672

        // while the status of each connection isn't open, ratchet through the handshake steps
        let mut a_state = ConnectionState::Uninitialized;
        let mut b_state = ConnectionState::Uninitialized;
        loop {
            // TODO: break out of loop after some # of iterations if we're stuck
            println!("a_state: {:?}, b_state: {:?}", a_state, b_state);
            match (a_state, b_state) {
                // send the Init message to chain a (source)
                (ConnectionState::Uninitialized, ConnectionState::Uninitialized) => {
                    // First part of handshake is ConnectionOpenInit
                    let plan = {
                        let ibc_msg = IbcRelay::ConnectionOpenInit(MsgConnectionOpenInit {
                            client_id_on_a: self.chain_a_ibc.client_id.clone(),
                            counterparty: self.chain_a_ibc.counterparty.clone(),
                            version: Some(self.chain_a_ibc.version.clone()),
                            delay_period: Duration::from_secs(1),
                            signer: self.chain_a_ibc.signer.clone(),
                        })
                        .into();
                        TransactionPlan {
                            actions: vec![ibc_msg],
                            // Now fill out the remaining parts of the transaction needed for verification:
                            memo: None,
                            detection_data: None, // We'll set this automatically below
                            transaction_parameters: TransactionParameters {
                                chain_id: format!("{}-a", TestNode::<()>::CHAIN_ID).to_string(),
                                ..Default::default()
                            },
                        }
                    };
                    let tx = self.chain_a_ibc.client.witness_auth_build(&plan).await?;

                    // Execute the transaction, applying it to the chain state.
                    let pre_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();
                    self.chain_a_ibc
                        .node
                        .block()
                        .with_data(vec![tx.encode_to_vec()])
                        .execute()
                        .await?;
                    let post_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();

                    // validate the connection state is now "init"
                    {
                        // Connection should not exist pre-commit
                        assert!(pre_tx_snapshot
                            .get_connection(&self.chain_a_ibc.connection_id)
                            .await?
                            .is_none(),);

                        // Post-commit, the connection should be in the "init" state.
                        let connection = post_tx_snapshot
                            .get_connection(&self.chain_a_ibc.connection_id)
                            .await?
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "no connection with the specified ID {} exists",
                                    &self.chain_a_ibc.connection_id
                                )
                            })?;

                        assert_eq!(connection.state.clone(), ConnectionState::Init);

                        self.chain_a_ibc.connection = Some(connection.clone());

                        // update a_state/b_state
                        a_state = connection.state;
                    }
                }

                // send the Try message to chain a (source)
                (ConnectionState::Uninitialized, ConnectionState::Init)
                | (ConnectionState::Init, ConnectionState::Init) => {
                    // First, an UpdateClient is sent to chain a (source) with the latest
                    // height for chain b (dst)
                    // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1010
                    // https://github.com/penumbra-zone/hermes/blob/main/crates/relayer/src/foreign_client.rs#L1144
                    // Fetch the consensus state on B and its associated proof
                    let client_state_of_a_on_b_response = self
                        .chain_b_ibc
                        .ibc_client_query_client
                        .client_state(QueryClientStateRequest {
                            client_id: self.chain_a_ibc.client_id.to_string(),
                        })
                        .await?
                        .into_inner();
                    let client_state_of_a_on_b = client_state_of_a_on_b_response
                        .client_state
                        .ok_or_else(|| anyhow!("client state of A on B not found"))?;
                    let consensus_on_b_response = self
                        .chain_b_ibc
                        .ibc_client_query_client
                        .consensus_state(QueryConsensusStateRequest {
                            client_id: self.chain_a_ibc.client_id.to_string(),
                            revision_number: client_state_of_a_on_b_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_number,
                            revision_height: client_state_of_a_on_b_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_height,
                            latest_height: false,
                        })
                        .await;

                    // If the client already stores a consensus state for the target height,
                    // there is no need to update the client
                    if consensus_on_b_response.is_err() {
                        unimplemented!("NEED TO UPDATE ON A");
                    }
                    let consensus_on_b_response = consensus_on_b_response?.into_inner();

                    // We need to fetch the state of the client on B to include the proofs in the ConnectionOpenTry message
                    // This response contains the necessary proof of the client state being on a
                    // Hermes proof generation: https://github.com/penumbra-zone/hermes/blob/main/crates/relayer/src/chain/endpoint.rs#L387
                    let client_state_of_b_on_a_response = self
                        .chain_a_ibc
                        .ibc_client_query_client
                        .client_state(QueryClientStateRequest {
                            client_id: self.chain_b_ibc.client_id.to_string(),
                        })
                        .await?
                        .into_inner();
                    let client_state_of_b_on_a = client_state_of_b_on_a_response
                        .client_state
                        .ok_or_else(|| anyhow!("client state of B on A not found"))?;

                    // TODO: any verification needed of cs_of_b_on_a?

                    // Fetch the connection and its associated proof
                    let connection_on_a_response = self
                        .chain_a_ibc
                        .ibc_connection_query_client
                        .connection(QueryConnectionRequest {
                            connection_id: self.chain_a_ibc.connection_id.to_string(),
                        })
                        .await?
                        .into_inner();

                    // Fetch the consensus state and its associated proof
                    let consensus_on_a_response = self
                        .chain_a_ibc
                        .ibc_client_query_client
                        .consensus_state(QueryConsensusStateRequest {
                            client_id: self.chain_b_ibc.client_id.to_string(),
                            revision_number: client_state_of_b_on_a_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_number,
                            revision_height: client_state_of_b_on_a_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_height,
                            latest_height: false,
                        })
                        .await?
                        .into_inner();

                    // Then send the ConnectionOpenTry message
                    println!("client_state_of_b_on_a: {:?}", client_state_of_b_on_a);
                    let plan = {
                        let ibc_msg = IbcRelay::ConnectionOpenTry(MsgConnectionOpenTry {
                            counterparty: self.chain_a_ibc.counterparty.clone(),
                            delay_period: Duration::from_secs(1),
                            signer: self.chain_a_ibc.signer.clone(),
                            client_id_on_b: self.chain_b_ibc.client_id.clone(),
                            client_state_of_b_on_a: client_state_of_a_on_b,
                            // TODO: query these?
                            versions_on_a: vec![Version::default()],
                            proof_conn_end_on_a: MerkleProof::decode(
                                connection_on_a_response.proof.as_slice(),
                            )?,
                            proof_client_state_of_b_on_a: MerkleProof::decode(
                                client_state_of_b_on_a_response.proof.as_slice(),
                            )?,
                            proof_consensus_state_of_b_on_a: MerkleProof::decode(
                                consensus_on_a_response.proof.as_slice(),
                            )?,
                            proofs_height_on_a: client_state_of_b_on_a_response
                                .proof_height
                                .clone()
                                .expect("height")
                                .try_into()?,
                            // TODO: not sure what we want here
                            consensus_height_of_b_on_a: client_state_of_b_on_a_response
                                .proof_height
                                .expect("height")
                                .try_into()?,
                            proof_consensus_state_of_b: Some(MerkleProof::decode(
                                consensus_on_b_response.proof.as_slice(),
                            )?),
                            // deprecated
                            previous_connection_id: "".to_string(),
                        })
                        .into();
                        TransactionPlan {
                            actions: vec![ibc_msg],
                            // Now fill out the remaining parts of the transaction needed for verification:
                            memo: None,
                            detection_data: None, // We'll set this automatically below
                            transaction_parameters: TransactionParameters {
                                chain_id: self.chain_a_ibc.chain_id.clone(),
                                ..Default::default()
                            },
                        }
                    };
                    let tx = self.chain_a_ibc.client.witness_auth_build(&plan).await?;

                    // Execute the transaction, applying it to the chain state.
                    let pre_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();
                    self.chain_a_ibc
                        .node
                        .block()
                        .with_data(vec![tx.encode_to_vec()])
                        .execute()
                        .await?;
                    let post_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();

                    // validate the connection state is now "init"
                    {
                        // Connection should not exist pre-commit
                        assert!(pre_tx_snapshot
                            .get_connection(&self.chain_a_ibc.connection_id)
                            .await?
                            .is_none(),);

                        // Post-commit, the connection should be in the "init" state.
                        let connection = post_tx_snapshot
                            .get_connection(&self.chain_a_ibc.connection_id)
                            .await?
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "no connection with the specified ID {} exists",
                                    &self.chain_a_ibc.connection_id
                                )
                            })?;

                        assert_eq!(connection.state, ConnectionState::Init);

                        self.chain_a_ibc.connection = Some(connection);
                    }
                }

                // send the Try message to chain b (destination)
                (ConnectionState::Init, ConnectionState::Uninitialized) => {
                    // TODO: refactor this copy paste
                    // First, an UpdateClient is sent to chain b (source) with the latest
                    // height for chain a (dst)
                    // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1010
                    // https://github.com/penumbra-zone/hermes/blob/main/crates/relayer/src/foreign_client.rs#L1144
                    // Fetch the consensus state on A and its associated proof
                    let client_state_of_b_on_a_response = self
                        .chain_a_ibc
                        .ibc_client_query_client
                        .client_state(QueryClientStateRequest {
                            client_id: self.chain_b_ibc.client_id.to_string(),
                        })
                        .await?
                        .into_inner();
                    let client_state_of_b_on_a = client_state_of_b_on_a_response
                        .client_state
                        .ok_or_else(|| anyhow!("client state of B on A not found"))?;
                    let consensus_on_a_response = self
                        .chain_a_ibc
                        .ibc_client_query_client
                        .consensus_state(QueryConsensusStateRequest {
                            client_id: self.chain_b_ibc.client_id.to_string(),
                            revision_number: client_state_of_b_on_a_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_number,
                            revision_height: client_state_of_b_on_a_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_height,
                            latest_height: false,
                        })
                        .await;

                    // If the client already stores a consensus state for the target height,
                    // there is no need to update the client
                    if consensus_on_a_response.is_err() {
                        unimplemented!("NEED TO UPDATE ON B");
                    }
                    let consensus_on_a_response = consensus_on_a_response?.into_inner();

                    let client_state_of_a_on_b_response = self
                        .chain_b_ibc
                        .ibc_client_query_client
                        .client_state(QueryClientStateRequest {
                            client_id: self.chain_a_ibc.client_id.to_string(),
                        })
                        .await?
                        .into_inner();
                    let client_state_of_a_on_b = client_state_of_a_on_b_response
                        .client_state
                        .ok_or_else(|| anyhow!("client state of A on B not found"))?;

                    let connection_on_b_response = self
                        .chain_b_ibc
                        .ibc_connection_query_client
                        .connection(QueryConnectionRequest {
                            connection_id: self.chain_b_ibc.connection_id.to_string(),
                        })
                        .await?
                        .into_inner();

                    let consensus_on_b_response = self
                        .chain_b_ibc
                        .ibc_client_query_client
                        .consensus_state(QueryConsensusStateRequest {
                            client_id: self.chain_a_ibc.client_id.to_string(),
                            revision_number: client_state_of_a_on_b_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_number,
                            revision_height: client_state_of_a_on_b_response
                                .proof_height
                                .clone()
                                .unwrap()
                                .revision_height,
                            latest_height: false,
                        })
                        .await?
                        .into_inner();

                    // Then send the ConnectionOpenTry message
                    let client_state =
                        ClientState::decode(client_state_of_a_on_b.value.as_slice())?;
                    println!("client_state_of_a_on_b: {:?}", client_state);
                    self.chain_b_ibc.counterparty.connection_id =
                        Some(self.chain_a_ibc.connection_id.clone());
                    let plan = {
                        let ibc_msg = IbcRelay::ConnectionOpenTry(MsgConnectionOpenTry {
                            counterparty: self.chain_b_ibc.counterparty.clone(),
                            delay_period: Duration::from_secs(1),
                            signer: self.chain_b_ibc.signer.clone(),
                            client_id_on_b: self.chain_a_ibc.client_id.clone(),
                            client_state_of_b_on_a: client_state_of_b_on_a,
                            // TODO: query these?
                            versions_on_a: vec![Version::default()],
                            proof_conn_end_on_a: MerkleProof::decode(
                                connection_on_b_response.proof.as_slice(),
                            )?,
                            proof_client_state_of_b_on_a: MerkleProof::decode(
                                client_state_of_a_on_b_response.proof.as_slice(),
                            )?,
                            proof_consensus_state_of_b_on_a: MerkleProof::decode(
                                consensus_on_b_response.proof.as_slice(),
                            )?,
                            proofs_height_on_a: client_state_of_a_on_b_response
                                .proof_height
                                .clone()
                                .expect("height")
                                .try_into()?,
                            // TODO: not sure what we want here
                            consensus_height_of_b_on_a: client_state_of_a_on_b_response
                                .proof_height
                                .expect("height")
                                .try_into()?,
                            proof_consensus_state_of_b: Some(MerkleProof::decode(
                                consensus_on_a_response.proof.as_slice(),
                            )?),
                            // deprecated
                            previous_connection_id: "".to_string(),
                        })
                        .into();
                        TransactionPlan {
                            actions: vec![ibc_msg],
                            // Now fill out the remaining parts of the transaction needed for verification:
                            memo: None,
                            detection_data: None, // We'll set this automatically below
                            transaction_parameters: TransactionParameters {
                                chain_id: self.chain_b_ibc.chain_id.clone(),
                                ..Default::default()
                            },
                        }
                    };
                    let tx = self.chain_b_ibc.client.witness_auth_build(&plan).await?;

                    // Execute the transaction, applying it to the chain state.
                    let pre_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();
                    self.chain_b_ibc
                        .node
                        .block()
                        .with_data(vec![tx.encode_to_vec()])
                        .execute()
                        .await?;
                    let post_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();

                    // validate the connection state is now "init"
                    {
                        // Connection should not exist pre-commit
                        assert!(pre_tx_snapshot
                            .get_connection(&self.chain_b_ibc.connection_id)
                            .await?
                            .is_none(),);

                        // Post-commit, the connection should be in the "init" state.
                        let connection = post_tx_snapshot
                            .get_connection(&self.chain_b_ibc.connection_id)
                            .await?
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "no connection with the specified ID {} exists",
                                    &self.chain_b_ibc.connection_id
                                )
                            })?;

                        assert_eq!(connection.state, ConnectionState::Init);

                        self.chain_b_ibc.connection = Some(connection);
                    }
                }

                _ => unimplemented!("unimplemented"),
            }
        }

        Err(anyhow::anyhow!("unimplemented"))
    }

    async fn _create_clients(&mut self) -> Result<(), anyhow::Error> {
        // Each chain will need a client created corresponding to its IBC connection with the other chain:
        let plan = {
            let ibc_msg = IbcRelay::CreateClient(MsgCreateClient {
                signer: self.chain_a_ibc.signer.clone(),
                client_state: ibc_types::lightclients::tendermint::client_state::ClientState {
                    // Chain ID is for the counterparty
                    chain_id: self.chain_b_ibc.chain_id.clone().into(),
                    trust_level: TrustThreshold {
                        numerator: 1,
                        denominator: 3,
                    },
                    trusting_period: Duration::from_secs(120),
                    unbonding_period: Duration::from_secs(240),
                    max_clock_drift: Duration::from_secs(5),
                    latest_height: Height {
                        revision_number: 0,
                        revision_height: 1,
                    },
                    proof_specs: IBC_PROOF_SPECS.to_vec(),
                    upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
                    allow_update: AllowUpdate {
                        after_expiry: false,
                        after_misbehaviour: false,
                    },
                    frozen_height: None,
                }
                .into(),
                consensus_state:
                    ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
                        timestamp: Time::parse_from_rfc3339("2020-09-14T16:33:00Z")?,
                        // Believe we use the chain B merkle root here... TODO: confirm
                        root: self.merkle_root_b().await?,
                        next_validators_hash: Hash::None,
                    }
                    .into(),
            })
            .into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_a_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = self.chain_a_ibc.client.witness_auth_build(&plan).await?;

        // Create the client for chain B on chain A.
        self.chain_a_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;

        let plan = {
            let ibc_msg = IbcRelay::CreateClient(MsgCreateClient {
                signer: self.chain_b_ibc.signer.clone(),
                client_state: ibc_types::lightclients::tendermint::client_state::ClientState {
                    // Chain ID is for the counterparty
                    chain_id: self.chain_a_ibc.chain_id.clone().into(),
                    trust_level: TrustThreshold {
                        numerator: 1,
                        denominator: 3,
                    },
                    trusting_period: Duration::from_secs(120),
                    unbonding_period: Duration::from_secs(240),
                    max_clock_drift: Duration::from_secs(5),
                    latest_height: Height {
                        revision_number: 0,
                        revision_height: 1,
                    },
                    proof_specs: IBC_PROOF_SPECS.to_vec(),
                    upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
                    allow_update: AllowUpdate {
                        after_expiry: false,
                        after_misbehaviour: false,
                    },
                    frozen_height: None,
                }
                .into(),
                consensus_state:
                    ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
                        timestamp: Time::parse_from_rfc3339("2020-09-14T16:33:00Z")?,
                        // Believe we use the chain A merkle root here... TODO: confirm
                        root: self.merkle_root_a().await?,
                        next_validators_hash: Hash::None,
                    }
                    .into(),
            })
            .into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_b_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = self.chain_b_ibc.client.witness_auth_build(&plan).await?;

        // Create the client for chain A on chain B.
        self.chain_b_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;

        Ok(())
    }

    async fn handshake(&mut self) -> Result<(), anyhow::Error> {
        // Open a connection on each chain to the other chain.
        // This is accomplished by following the ICS-003 spec for connection handshakes.

        // The Clients need to be created on each chain prior to the handshake.
        self._create_clients().await?;
        // The handshake is a multi-step process, this call will ratchet through the steps.
        self._handshake().await?;

        Ok(())
    }
}

/// Exercises that the IBC happy path succeeds.
#[tokio::test]
async fn ibc_happy_path() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();

    // Set up some configuration for the two different chains we'll need to keep around.
    let chain_a_ibc = TestIbcData::new("a").await?;
    let chain_b_ibc = TestIbcData::new("b").await?;

    // The Relayer will handle IBC operations and manage state for the two test chains
    let mut relayer = TestRelayer {
        chain_a_ibc,
        chain_b_ibc,
    };

    // Perform the IBC handshake between the two chains.
    relayer.handshake().await?;

    Ok(()).tap(|_| drop(relayer)).tap(|_| drop(guard))
}
