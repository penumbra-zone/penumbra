use {
    anyhow::{Context as _, Result},
    cnidarium::{StateRead as _, TempStorage},
    common::{
        ibc_tests::{get_verified_genesis, MockRelayer, TestNodeWithIBC, ValidatorKeys},
        TempStorageExt as _,
    },
    ibc_proto::ibc::core::client::v1::{
        query_client::QueryClient as IbcClientQueryClient, QueryClientStateRequest,
    },
    ibc_types::{
        core::{
            client::{msgs::MsgCreateClient, ClientId, Height},
            commitment::{MerkleProof, MerkleRoot},
        },
        lightclients::tendermint::{
            client_state::{AllowUpdate, ClientState as TendermintClientState},
            TrustThreshold,
        },
        path::ClientStatePath,
        DomainType as _,
    },
    once_cell::sync::Lazy,
    penumbra_app::server::consensus::Consensus,
    penumbra_ibc::{IbcRelay, MerklePrefixExt, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS},
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::{
        cnidarium::v1::{
            query_service_client::QueryServiceClient as CnidariumQueryServiceClient,
            KeyValueRequest,
        },
        util::tendermint_proxy::v1::{
            tendermint_proxy_service_client::TendermintProxyServiceClient, GetBlockByHeightRequest,
        },
        DomainType, Message as _,
    },
    penumbra_test_subscriber::set_tracing_subscriber_with_env_filter,
    penumbra_transaction::{TransactionParameters, TransactionPlan},
    std::{str::FromStr as _, time::Duration},
    tap::{Tap, TapFallible as _},
    tendermint::Hash,
    tokio::time,
    tonic::transport::Channel,
};

/// The proof specs for the main store.
pub static MAIN_STORE_PROOF_SPEC: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![cnidarium::ics23_spec()]);

mod common;

fn set_tracing_subscriber() -> tracing::subscriber::DefaultGuard {
    let filter = "info,penumbra_app=info,penumbra_mock_consensus=trace,jmt=trace";
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(filter))
        .expect("should have a valid filter directive");
    set_tracing_subscriber_with_env_filter(filter)
}

// Snapshot version is used as the revision height in the IBC client_state query.
// Therefore we need to validate that the snapshot revision is the same as the
// Mock Tendermint height.
#[tokio::test]
async fn mocktendermint_snapshot_versions() -> anyhow::Result<()> {
    let _guard = set_tracing_subscriber();

    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    let proxy = penumbra_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();
    let mut node = {
        let genesis = get_verified_genesis()?;
        let consensus = Consensus::new(storage.clone());
        // Hardcoded keys for each chain for test reproducibility:
        let sk_a = ed25519_consensus::SigningKey::from([0u8; 32]);
        let vk_a = sk_a.verification_key();
        let keys = (sk_a, vk_a);
        // let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .with_keys(vec![keys])
            .single_validator()
            .with_tendermint_genesis(genesis)
            .on_block(proxy.on_block_callback())
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };
    let grpc_url = "http://127.0.0.1:8081" // see #4517
        .parse::<url::Url>()?
        .tap(|url| tracing::debug!(%url, "parsed grpc url"));
    // Spawn the node's RPC server.
    let _rpc_server = {
        let make_svc =
            penumbra_app::rpc::router(&storage, proxy, false /*enable_expensive_rpc*/)?
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
    let channel = Channel::from_shared(grpc_url.to_string())
        .with_context(|| "could not parse node URI")?
        .connect()
        .await
        .with_context(|| "could not connect to grpc server")
        .tap_err(|error| tracing::error!(?error, "could not connect to grpc server"))?;
    let mut tendermint_proxy_service_client = TendermintProxyServiceClient::new(channel.clone());

    assert_eq!(u64::from(*node.height()), 0u64);

    // we're still on block 0, execute a block 1 with no transactions.
    node.block().execute().await?;

    // block header 1 has now been created.
    let block_1: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                // get block height 1
                height: 1.into(),
            })
            .await?
            .into_inner();

    assert_eq!(u64::from(*node.height()), 1u64);

    // we know the block 1 app_hash should always be 5c94f2eabd29ac36f5be7f812a586b5dd44c10d586d2bb1a18e3679801d1b5dd
    // for the test genesis data
    println!("block 1: {:?}", block_1);
    assert_eq!(
        hex::decode("5c94f2eabd29ac36f5be7f812a586b5dd44c10d586d2bb1a18e3679801d1b5dd")?,
        block_1.block.unwrap().header.unwrap().app_hash
    );

    let snapshot = storage.latest_snapshot();
    let storage_revision_height = snapshot.version();

    let saved_height = node.height().clone();
    // JMT storage revision height should always match the mock tendermint height
    assert_eq!(u64::from(saved_height), storage_revision_height);
    // store the root of storage at this height for later verification
    let saved_storage_root = snapshot.root_hash().await?;
    println!(
        "storage height is {} and storage root is {}",
        storage_revision_height,
        hex::encode(saved_storage_root.0)
    );

    // execute a few blocks
    node.block().execute().await?;
    node.block().execute().await?;
    node.block().execute().await?;

    let proof_block: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                // Use the height from earlier
                height: saved_height.into(),
            })
            .await?
            .into_inner();

    // We fetched the block associated with the height from earlier
    // and can validate that its app hash in the block header
    // matches the value we got directly from storage earlier:
    assert_eq!(
        proof_block.block.clone().unwrap().header.unwrap().app_hash,
        saved_storage_root.0,
        "block app hash {} should match storage root {}",
        hex::encode(proof_block.block.unwrap().header.unwrap().app_hash),
        hex::encode(saved_storage_root.0)
    );

    Ok(())
}

/// Validates the cometbft mock behavior against real cometbft
/// using the same genesis data.
#[tokio::test]
async fn cometbft_mock_verification() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let _guard = set_tracing_subscriber();

    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    let mut node = {
        let genesis = get_verified_genesis()?;
        let consensus = Consensus::new(storage.clone());
        // let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_tendermint_genesis(genesis)
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // This is the app hash cometBFT sees in the InitChain message
    assert_eq!(
        node.last_app_hash_hex().to_lowercase(),
        "5c94f2eabd29ac36f5be7f812a586b5dd44c10d586d2bb1a18e3679801d1b5dd"
    );

    node.block().execute().await?;
    node.block().execute().await?;
    node.block().execute().await?;

    Ok(())
}

#[tokio::test]
async fn verify_storage_proof_simple() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = set_tracing_subscriber();

    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    let start_time = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    let proxy = penumbra_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();
    let mut node = {
        let genesis = get_verified_genesis()?;
        let consensus = Consensus::new(storage.clone());
        // let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_tendermint_genesis(genesis)
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

    println!("verified directly from storage");

    // now verify the proof retrieved via a gRPC call
    let grpc_url = "http://127.0.0.1:8081" // see #4517
        .parse::<url::Url>()?
        .tap(|url| tracing::debug!(%url, "parsed grpc url"));
    // Spawn the node's RPC server.
    let _rpc_server = {
        let make_svc =
            penumbra_app::rpc::router(&storage, proxy, false /*enable_expensive_rpc*/)?
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
    // height 2
    // WRONG vvv these don't match what's in the block headers
    let node_last_app_hash = node.last_app_hash();
    println!(
        "making IBC client state request at height {} and hash {}",
        latest_height,
        // e0c071d4b2198c7e5f9fdee7d6618bf36ea75fdecd56df315ba2ae87b9a50718 (height 3 header app_hash)
        hex::encode(node_last_app_hash)
    );
    let ibc_client_state_response = ibc_client_query_client
        .client_state(QueryClientStateRequest {
            client_id: "07-tendermint-0".to_string(),
        })
        .await?
        .into_inner();

    let ibc_proof = MerkleProof::decode(ibc_client_state_response.clone().proof.as_slice())?;
    let ibc_value = ibc_client_state_response.client_state.unwrap();

    // let cs = ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
    //     ibc_value.clone(),
    // )?;
    // println!("client state: {:?}", cs);
    // // let cs2 = ibc_types::lightclients::tendermint::client_state::ClientState::try_from(Any {
    // //     type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
    // //     value: value.clone().into(),
    // // })?;
    // let client_state = ibc_proto::google::protobuf::Any::decode(value.as_ref())?;
    // let cs2 = ibc_proto::ibc::lightclients::tendermint::v1::ClientState::decode(
    //     &*client_state.value.clone(),
    // )?;
    // let cs3 =
    //     ibc_types::lightclients::tendermint::client_state::ClientState::try_from(client_state)?;
    // println!("client state2: {:?}", cs2);
    // println!("client state3: {:?}", cs3);

    // let client_state = ibc_proto::google::protobuf::Any::decode(value.as_ref())?;
    // let cs1 = ibc_proto::ibc::lightclients::tendermint::v1::ClientState::decode(&*client.value)?;
    // let client_state1 = TendermintClientState::try_from(cs1.clone())?;

    assert_eq!(ibc_value.encode_to_vec(), value);

    // We should be able to get the block from the proof_height associated with
    // the proof and use the app_hash as the jmt root and succeed in proving:
    let proof_block: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
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
    // The node height when we directly retrieved the last app hash
    // should match the proof height
    assert_eq!(
        ibc_client_state_response
            .proof_height
            .clone()
            .unwrap()
            .revision_height,
        u64::from(latest_height)
    );
    // the proof block's app hash should match
    // assert_eq!(
    //     node_last_app_hash,
    //     proof_block.block.clone().unwrap().header.unwrap().app_hash,
    //     "node claimed app hash for height {} was {}, however block header contained {}",
    //     node_height,
    //     hex::encode(node_last_app_hash),
    //     hex::encode(proof_block.block.clone().unwrap().header.unwrap().app_hash)
    // );
    println!(
        "proof height: {} proof_block_root: {:?}",
        ibc_client_state_response
            .proof_height
            .unwrap()
            .revision_height,
        hex::encode(proof_block.block.clone().unwrap().header.unwrap().app_hash)
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

/// Exercises that the IBC handshake succeeds.
#[tokio::test]
async fn ibc_handshake() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();

    // Fixed start times (both chains start at the same time to avoid unintended timeouts):
    let start_time_a = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;
    let start_time_b = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    // But chain B will be 39 blocks ahead of chain A, so offset chain A's
    // start time so they match:
    // let start_time_b = start_time_a.checked_sub(39 * block_duration).unwrap();

    // Hardcoded keys for each chain for test reproducibility:
    let vkeys_a = ValidatorKeys::from_seed([0u8; 32]);
    let vkeys_b = ValidatorKeys::from_seed([1u8; 32]);
    let sk_a = vkeys_a.validator_cons_sk.ed25519_signing_key().unwrap();
    let sk_b = vkeys_b.validator_cons_sk.ed25519_signing_key().unwrap();

    let ska = ed25519_consensus::SigningKey::try_from(sk_a.as_bytes())?;
    let skb = ed25519_consensus::SigningKey::try_from(sk_b.as_bytes())?;
    let keys_a = (ska.clone(), ska.verification_key());
    let keys_b = (skb.clone(), skb.verification_key());

    // Set up some configuration for the two different chains we'll need to keep around.
    let mut chain_a_ibc = TestNodeWithIBC::new("a", start_time_a, keys_a).await?;
    let mut chain_b_ibc = TestNodeWithIBC::new("b", start_time_b, keys_b).await?;

    chain_a_ibc.node.block().execute().await?;
    chain_b_ibc.node.block().execute().await?;
    // The two chains can't IBC handshake during the first block, let's fast forward
    // them both a few.
    // for _ in 0..3 {
    //     chain_a_ibc.node.block().execute().await?;
    // }
    // // Do them each a different # of blocks to make sure the heights don't get confused.
    // for _ in 0..42 {
    // chain_b_ibc.node.block().execute().await?;
    // }

    // The chains should be at the same time:
    assert_eq!(chain_a_ibc.node.timestamp(), chain_b_ibc.node.timestamp());
    // But their block heights should be different:
    // assert_ne!(
    //     chain_a_ibc.get_latest_height().await?,
    //     chain_b_ibc.get_latest_height().await?,
    // );

    // The Relayer will handle IBC operations and manage state for the two test chains
    let mut relayer = MockRelayer {
        chain_a_ibc,
        chain_b_ibc,
    };

    // Perform the IBC connection handshake between the two chains.
    // TODO: some testing of failure cases of the handshake process would be good
    // The Clients need to be created on each chain prior to the handshake.
    relayer._create_clients().await?;

    relayer._sync_chains().await?;

    relayer._build_and_send_connection_open_init().await?;

    relayer._sync_chains().await?;

    relayer._build_and_send_connection_open_try().await?;

    Ok(()).tap(|_| drop(relayer)).tap(|_| drop(guard))
}

/// Tests of the mock IBC relayer
#[tokio::test]
async fn ibc_granular_tests() -> anyhow::Result<()> {
    // let gk_hex = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29".to_string();
    // let gk = hex::decode(gk_hex).unwrap();
    // assert_eq!(gk.len(), 32);
    // let pbgk = ProtoGovernanceKey { gk: gk.clone() };

    // let governance_key: GovernanceKey = pbgk.try_into().unwrap();
    // gov key needs to be a
    // decaf377::Encoding
    // aka it needs to actually be a code point
    // maybe better to just generate a real key
    // or use a preexisting test one
    // println!("govvy : {:?}", governance_key);

    // let governance_key: penumbra_stake::GovernanceKey =
    //     penumbra_proto::core::keys::v1::GovernanceKey {
    //         gk: [0u8; 32].to_vec(),
    //     }
    //     .try_into()
    //     .unwrap();
    // Install a test logger, and acquire some temporary storage.
    let _guard = common::set_tracing_subscriber();

    let start_time = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    let vkeys_a = ValidatorKeys::from_seed([0u8; 32]);
    let sk_a = vkeys_a.validator_cons_sk.ed25519_signing_key().unwrap();
    let vkeys_b = ValidatorKeys::from_seed([1u8; 32]);
    let sk_b = vkeys_b.validator_cons_sk.ed25519_signing_key().unwrap();

    // TODO: make testnodewithibc work with a validatorkeys struct
    let ska = ed25519_consensus::SigningKey::try_from(sk_a.as_bytes())?;
    let keys_a = (ska.clone(), ska.verification_key());
    let skb = ed25519_consensus::SigningKey::try_from(sk_b.as_bytes())?;
    let keys_b = (skb.clone(), skb.verification_key());
    // Set up some configuration for the two different chains we'll need to keep around.
    let mut chain_a_ibc = TestNodeWithIBC::new("a", start_time.clone(), keys_a).await?;
    let mut chain_b_ibc = TestNodeWithIBC::new("b", start_time, keys_b).await?;

    // The two chains can't IBC handshake during the first block, let's fast forward
    // them both 1.
    chain_a_ibc.node.block().execute().await?;
    chain_b_ibc.node.block().execute().await?;

    // The Relayer will handle IBC operations and manage state for the two test chains
    let mut relayer = MockRelayer {
        chain_a_ibc,
        chain_b_ibc,
    };

    relayer._create_clients().await?;
    // relayer._sync_chains().await?;
    // relayer._build_and_send_connection_open_init().await?;
    relayer._sync_chains().await?;

    relayer.chain_a_ibc.node.block().execute().await?;
    relayer.chain_b_ibc.node.block().execute().await?;

    // chain A will have created a new client for chain B
    // we should be able to retrieve the connection, check its
    // details, and verify the proof that was generated for it
    let client_state_of_b_on_a_response = relayer
        .chain_a_ibc
        .ibc_client_query_client
        .client_state(QueryClientStateRequest {
            client_id: relayer.chain_b_ibc.client_id.to_string(),
        })
        .await?
        .into_inner();

    let client_state = TendermintClientState::try_from(
        client_state_of_b_on_a_response
            .client_state
            .clone()
            .unwrap(),
    )?;
    println!(
        "FETCHED CLIENT STATE OF B ON A AT B HEIGHT {}, A HEIGHT {}, CLIENT STATE HEIGHT {}",
        relayer.chain_a_ibc.get_latest_height().await?,
        relayer.chain_b_ibc.get_latest_height().await?,
        client_state.latest_height()
    );

    // Decode the proofs...
    let proof_client_state_of_b_on_a =
        MerkleProof::decode(client_state_of_b_on_a_response.clone().proof.as_slice())?;

    // Validate the proofs
    let proof_specs = IBC_PROOF_SPECS.to_vec();
    // Initial path is the key...
    let csp = ClientStatePath(ClientId::from_str(&relayer.chain_b_ibc.client_id.as_str())?);
    let prefix = &IBC_COMMITMENT_PREFIX;

    // The root is the current root of chain a at the height of the proof
    let proof_height = client_state_of_b_on_a_response
        .proof_height
        .clone()
        .unwrap();
    let chain_a_proof_block: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        relayer
            .chain_a_ibc
            .tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                height: proof_height.revision_height.try_into()?,
            })
            .await?
            .into_inner();
    let proof_root = MerkleRoot {
        hash: chain_a_proof_block.block.unwrap().header.unwrap().app_hash,
    };
    println!(
        "proof root for height {}: {}",
        proof_height.revision_height,
        hex::encode(proof_root.hash.clone())
    );
    let merkle_path = prefix.apply(vec![csp.to_string()]);

    let unproven_client_state = client_state_of_b_on_a_response
        .clone()
        .client_state
        .unwrap();

    // Now retrieving the client state directly from storage should succeed:
    let snapshot = relayer.chain_a_ibc.storage.clone().latest_snapshot();

    let client_id = "07-tendermint-0".to_string();
    let key = IBC_COMMITMENT_PREFIX
        .apply_string(ClientStatePath(ClientId::from_str(&client_id)?).to_string())
        .as_bytes()
        .to_vec();
    let (cs_opt, cnid_proof) = snapshot
        .get_with_proof(key.clone())
        .await
        .map_err(|e| tonic::Status::aborted(format!("couldn't get client: {e}")))?;

    let client_state = cs_opt
        .clone()
        .map(|cs_opt| ibc_proto::google::protobuf::Any::decode(cs_opt.as_ref()))
        .transpose()
        .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

    // The proven version should also be present
    let client = client_state.unwrap();
    let unproven_client = unproven_client_state.clone();

    // assert_eq!(client.value, cs_opt.clone().unwrap());
    let cs1 = ibc_proto::ibc::lightclients::tendermint::v1::ClientState::decode(&*client.value)?;
    let client_state1 = TendermintClientState::try_from(cs1.clone())?;
    let cs2 =
        ibc_proto::ibc::lightclients::tendermint::v1::ClientState::decode(&*unproven_client.value)?;
    let client_state2 = TendermintClientState::try_from(cs2.clone())?;
    println!("client state 1 (from cnidarium): {:?}", client_state1);
    println!(
        "client state 2 (from the client_state RPC): {:?}",
        client_state2
    );
    assert_eq!(cs1, cs2);

    // The response from the RPC should be the same as the client state from storage
    assert_eq!(
        cs_opt.clone().unwrap(),
        unproven_client_state.encode_to_vec()
    );

    // Test the signature directly from cnid against the client state directly
    // from cnid
    println!("cnid_proof check");
    let cnid_root = snapshot.root_hash().await?;
    cnid_proof.verify_membership(
        &proof_specs,
        MerkleRoot {
            hash: cnid_root.0.to_vec(),
        },
        merkle_path.clone(),
        cs_opt.unwrap(),
        0,
    )?;

    println!("rpc proof check");
    // The unproven client state requires a root of the proof height
    // HOWEVER right now it seems like the root you get (from the GetBlockByHeight
    // app_hash ultimately) is one behind the root it was actually retrieved with.

    // statement of the problem:
    // the storage version (height)
    // is off by one with the tendermint header
    // height.
    // the root returned in the getblocksbyheight header
    // is for one height too low, i.e. for height 3 it returns the height 2
    // root.
    // are we wrong when creating the header?
    // when we are creating the client, the node's merkle root is correct.
    // then we call commit to commit that block.
    proof_client_state_of_b_on_a.verify_membership(
        &proof_specs,
        proof_root,
        merkle_path.clone(),
        unproven_client_state.encode_to_vec(),
        0,
    )?;

    // _build_and_send_update_client(&mut relayer.chain_a_ibc, &mut relayer.chain_b_ibc).await?;

    Ok(())
}

#[tokio::test]
async fn real_cometbft_tests() -> Result<()> {
    let grpc_url = "http://127.0.0.1:8080"
        .parse::<url::Url>()?
        .tap(|url| tracing::debug!(%url, "parsed grpc url"));

    let channel = Channel::from_shared(grpc_url.to_string())
        .with_context(|| "could not parse node URI")?
        .connect()
        .await
        .with_context(|| "could not connect to grpc server")
        .tap_err(|error| tracing::error!(?error, "could not connect to grpc server"))?;

    let mut tendermint_proxy_service_client = TendermintProxyServiceClient::new(channel.clone());

    let b = tendermint_proxy_service_client
        .get_block_by_height(GetBlockByHeightRequest { height: 333 })
        .await?
        .into_inner();

    println!(
        "block 333 app_hash: {}, last_block_id: {:?}, height: {} last_commit_hash: {}",
        hex::encode(b.clone().block.unwrap().header.unwrap().app_hash),
        b.clone().block.unwrap().header.unwrap().last_block_id,
        b.clone().block.unwrap().header.unwrap().height,
        hex::encode(b.clone().block.unwrap().header.unwrap().last_commit_hash),
    );

    println!("BLOCK: {:?}", b);

    Ok(())
}
