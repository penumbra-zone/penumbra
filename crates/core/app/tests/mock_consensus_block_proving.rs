use {
    anyhow::Context as _,
    cnidarium::proto::v1::{
        query_service_client::QueryServiceClient as CnidariumQueryServiceClient, KeyValueRequest,
    },
    cnidarium::{StateRead as _, TempStorage},
    common::{BuilderExt as _, TempStorageExt as _},
    ibc_proto::ibc::core::client::v1::{
        query_client::QueryClient as IbcClientQueryClient, QueryClientStateRequest,
    },
    ibc_types::{
        core::{
            client::{msgs::MsgCreateClient, ClientId, Height},
            commitment::{MerkleProof, MerkleRoot},
        },
        lightclients::tendermint::{client_state::AllowUpdate, TrustThreshold},
        path::ClientStatePath,
        DomainType as _,
    },
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_ibc::{IbcRelay, MerklePrefixExt as _, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS},
    penumbra_sdk_keys::test_keys,
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::{
        util::tendermint_proxy::v1::{
            tendermint_proxy_service_client::TendermintProxyServiceClient, GetBlockByHeightRequest,
        },
        DomainType, Message as _,
    },
    penumbra_sdk_test_subscriber::set_tracing_subscriber,
    penumbra_sdk_transaction::{TransactionParameters, TransactionPlan},
    std::{str::FromStr as _, time::Duration},
    tap::{Tap as _, TapFallible as _},
    tendermint::Hash,
    tokio::time::{self},
    tonic::transport::Channel,
};

mod common;

#[tokio::test]
async fn verify_storage_proof_simple() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = set_tracing_subscriber();

    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    let start_time = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    let proxy = penumbra_sdk_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();
    let mut node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
        let consensus = Consensus::new(storage.clone());
        // let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .on_block(proxy.on_block_callback())
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    let client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_inner_storage(storage.clone())
        .await?
        .tap(
            |c| tracing::info!(client.notes = %c.notes.len(), "mock client synced to test storage"),
        );

    node.block().execute().await?;

    // Force the node to write an IBC client status into storage
    // so we can retrieve with a proof:
    let plan = {
        let ibc_msg = IbcRelay::CreateClient(MsgCreateClient {
            signer: "test".to_string(),
            client_state: ibc_types::lightclients::tendermint::client_state::ClientState {
                chain_id: TestNode::<()>::CHAIN_ID.to_string().into(),
                trust_level: TrustThreshold {
                    numerator: 1,
                    denominator: 3,
                },
                trusting_period: Duration::from_secs(120_000),
                unbonding_period: Duration::from_secs(240_000),
                max_clock_drift: Duration::from_secs(5),
                latest_height: Height {
                    revision_height: 55,
                    revision_number: 0,
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
            consensus_state: ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
                timestamp: start_time,
                // These values don't matter since we are only checking the proof
                // of the client state.
                root: MerkleRoot {
                    hash: vec![0u8; 32],
                },
                next_validators_hash: Hash::Sha256([0u8; 32]),
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
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
    };
    let tx = client.witness_auth_build(&plan).await?;
    let client_id = "07-tendermint-0".to_string();
    let key = IBC_COMMITMENT_PREFIX
        .apply_string(ClientStatePath(ClientId::from_str(&client_id)?).to_string())
        .as_bytes()
        .to_vec();

    // Create the fake client
    node.block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;

    // Now retrieving the client state directly from storage should succeed:
    let snapshot = storage.latest_snapshot();

    let unproven = snapshot
        .get_raw(&String::from_utf8(key.clone())?)
        .await?
        .expect("present in storage");

    // The unproven version should be present
    assert!(!unproven.is_empty());

    let (cnid_client_state, cnid_proof) = snapshot
        .get_with_proof(key.clone())
        .await
        .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?;

    // The proven version should also be present
    let cnid_client_state = cnid_client_state.unwrap();

    // The proven version should be the same as the unproven.
    assert_eq!(cnid_client_state, unproven);

    // Common proof parameters:
    let proof_specs = IBC_PROOF_SPECS.to_vec();
    // The root will be the latest jmt hash.
    let latest_root = storage.latest_snapshot().root_hash().await.unwrap();
    let root = MerkleRoot {
        hash: latest_root.0.to_vec(),
    };
    // Initial path is the key...
    let csp = ClientStatePath(ClientId::from_str(&client_id)?);
    let prefix = &IBC_COMMITMENT_PREFIX;
    // With the prefix applied:
    let merkle_path = prefix.apply(vec![csp.to_string()]);

    // Verify the proof against the results from calling get_with_proof.
    cnid_proof.verify_membership(
        &proof_specs,
        root.clone(),
        merkle_path.clone(),
        cnid_client_state.clone(),
        0,
    )?;

    // now verify the proof retrieved via a gRPC call
    let grpc_url = "http://127.0.0.1:8081" // see #4517
        .parse::<url::Url>()?
        .tap(|url| tracing::debug!(%url, "parsed grpc url"));
    // Spawn the node's RPC server.
    let _rpc_server = {
        let make_svc =
            penumbra_sdk_app::rpc::routes(&storage, proxy, false /*enable_expensive_rpc*/)?
                .into_axum_router()
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
    let channel = Channel::from_shared(grpc_url.to_string())
        .with_context(|| "could not parse node URI")?
        .connect()
        .await
        .with_context(|| "could not connect to grpc server")
        .tap_err(|error| tracing::error!(?error, "could not connect to grpc server"))?;
    let mut cnidarium_client = CnidariumQueryServiceClient::new(channel.clone());
    let mut ibc_client_query_client = IbcClientQueryClient::new(channel.clone());
    let mut tendermint_proxy_service_client = TendermintProxyServiceClient::new(channel.clone());
    let kvr = cnidarium_client
        .key_value(tonic::Request::new(KeyValueRequest {
            key: String::from_utf8(key.clone()).unwrap(),
            proof: true,
        }))
        .await?
        .into_inner();

    let proof = kvr.proof.unwrap().try_into()?;
    let value = kvr.value.unwrap().value;

    // The proof from cnidarium and from the RPC should be the same since nothing has
    // happened on-chain since the cnidarium proof was generated.
    assert_eq!(cnid_proof, proof);
    // Same for the values.
    assert_eq!(value, cnid_client_state);

    proof.verify_membership(
        &proof_specs,
        root.clone(),
        merkle_path.clone(),
        value.clone(),
        0,
    )?;

    let snapshot = storage.latest_snapshot();
    let storage_revision_height = snapshot.version();

    let latest_height = node.height().clone();
    assert_eq!(u64::from(latest_height), storage_revision_height);

    // Try fetching the client state via the IBC API
    let node_last_app_hash = node.last_app_hash().to_vec();
    tracing::debug!(
        "making IBC client state request at height {} and hash {}",
        latest_height,
        hex::encode(&node_last_app_hash)
    );
    let ibc_client_state_response = ibc_client_query_client
        .client_state(QueryClientStateRequest {
            client_id: "07-tendermint-0".to_string(),
        })
        .await?
        .into_inner();

    assert!(
        ibc_client_state_response.client_state.as_ref().is_some()
            && !ibc_client_state_response
                .client_state
                .as_ref()
                .unwrap()
                .value
                .is_empty()
    );

    let ibc_proof = MerkleProof::decode(ibc_client_state_response.clone().proof.as_slice())?;
    let ibc_value = ibc_client_state_response.client_state.unwrap();

    assert_eq!(ibc_value.encode_to_vec(), value);

    // The current height of the node should be one behind the proof height.
    assert_eq!(
        u64::from(latest_height) + 1,
        ibc_client_state_response
            .proof_height
            .clone()
            .unwrap()
            .revision_height
    );

    let proof_block: penumbra_sdk_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                height: ibc_client_state_response
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_height
                    .try_into()?,
            })
            .await?
            .into_inner();

    // The proof block should be nonexistent because we haven't finalized the in-progress
    // block yet.
    assert!(proof_block.block.is_none());

    // Execute a block to finalize the proof block.
    node.block().execute().await?;

    // We should be able to get the block from the proof_height associated with
    // the proof and use the app_hash as the jmt root and succeed in proving:
    let proof_block: penumbra_sdk_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                height: ibc_client_state_response
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_height
                    .try_into()?,
            })
            .await?
            .into_inner();

    // The proof height of the ibc response should be the same as the height of the proof block
    assert_eq!(
        ibc_client_state_response
            .proof_height
            .clone()
            .unwrap()
            .revision_height,
        proof_block.block.clone().unwrap().header.unwrap().height as u64
    );

    let proof_block_root = MerkleRoot {
        hash: proof_block.block.unwrap().header.unwrap().app_hash,
    };
    ibc_proof
        .verify_membership(
            &proof_specs,
            proof_block_root,
            merkle_path,
            ibc_value.encode_to_vec(),
            0,
        )
        .expect("the ibc proof should validate against the root of the proof_height's block");

    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
