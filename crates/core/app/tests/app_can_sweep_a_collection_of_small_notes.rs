use {
    anyhow::Context,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{AppState, Content},
        server::consensus::Consensus,
    },
    penumbra_sdk_asset::{STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM},
    penumbra_sdk_keys::{keys::AddressIndex, test_keys},
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::{
        view::v1::{
            view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
            StatusRequest, StatusResponse,
        },
        DomainType,
    },
    penumbra_sdk_shielded_pool::genesis::Allocation,
    penumbra_sdk_view::ViewClient,
    penumbra_sdk_wallet::plan::SWEEP_COUNT,
    rand_core::OsRng,
    std::ops::Deref,
    tap::{Tap, TapFallible},
};

mod common;

/// The number of notes placed in the test wallet at genesis.
//  note: when debugging, it can help to set this to a lower value.
const COUNT: usize = SWEEP_COUNT + 1;

/// Exercises that the app can process a "sweep", consolidating small notes.
//  NB: a multi-thread runtime is needed to run both the view server and its client.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Flaked in #4626"]
async fn app_can_sweep_a_collection_of_small_notes() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber_with_env_filter("info".into());
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Instantiate a mock tendermint proxy, which we will connect to the test node.
    let proxy = penumbra_sdk_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();

    // Define allocations to the test address, as many small notes.
    let allocations = {
        let dust = Allocation {
            raw_amount: 1_u128.into(),
            raw_denom: STAKING_TOKEN_DENOM.deref().base_denom().denom,
            address: test_keys::ADDRESS_0.to_owned(),
        };
        std::iter::repeat(dust).take(COUNT).collect()
    };

    // Define our application state, and start the test node.
    let mut test_node = {
        let content = Content {
            chain_id: TestNode::<()>::CHAIN_ID.to_string(),
            shielded_pool_content: penumbra_sdk_shielded_pool::genesis::Content {
                allocations,
                ..Default::default()
            },
            ..Default::default()
        };
        let app_state = AppState::Content(content);
        let app_state = serde_json::to_vec(&app_state).unwrap();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .app_state(app_state)
            .on_block(proxy.on_block_callback())
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(
            |c| tracing::info!(client.notes = %c.notes.len(), "mock client synced to test storage"),
        );

    // Jump ahead a few blocks.
    test_node
        .fast_forward(10)
        .tap(|_| tracing::debug!("fast forwarding past genesis"))
        .await?;

    let grpc_url = "http://127.0.0.1:8081" // see #4517
        .parse::<url::Url>()?
        .tap(|url| tracing::debug!(%url, "parsed grpc url"));

    // Spawn the server-side view server.
    {
        let make_svc = penumbra_sdk_app::rpc::routes(
            storage.as_ref(),
            proxy,
            false, /*enable_expensive_rpc*/
        )?
        .into_axum_router()
        .layer(tower_http::cors::CorsLayer::permissive())
        .into_make_service()
        .tap(|_| tracing::debug!("initialized rpc service"));
        let [addr] = grpc_url
            .socket_addrs(|| None)?
            .try_into()
            .expect("grpc url can be turned into a socket address");
        let server = axum_server::bind(addr).serve(make_svc);
        tokio::spawn(async { server.await.expect("grpc server returned an error") })
            .tap(|_| tracing::debug!("grpc server is running"))
    };

    // Spawn the client-side view server...
    let view_server = {
        penumbra_sdk_view::ViewServer::load_or_initialize(
            None::<&camino::Utf8Path>,
            None::<&camino::Utf8Path>,
            &*test_keys::FULL_VIEWING_KEY,
            grpc_url,
        )
        .await
        // TODO(kate): the goal is to communicate with the `ViewServiceServer`.
        .map(ViewServiceServer::new)
        .context("initializing view server")?
    };

    // Create a view client, and get the test wallet's notes.
    let mut view_client = ViewServiceClient::new(view_server);

    // Sync the view client to the chain.
    {
        use futures::StreamExt;
        let mut status_stream = ViewClient::status_stream(&mut view_client).await?;
        while let Some(status) = status_stream.next().await.transpose()? {
            tracing::info!(?status, "view client received status stream response");
        }
        // Confirm that the status is as expected: synced up to the 11th block.
        let status = view_client.status(StatusRequest {}).await?.into_inner();
        debug_assert_eq!(
            status,
            StatusResponse {
                full_sync_height: 10,
                partial_sync_height: 10,
                catching_up: false,
            }
        );
    }

    client.sync_to_latest(storage.latest_snapshot()).await?;
    debug_assert_eq!(
        client
            .notes_by_asset(STAKING_TOKEN_ASSET_ID.deref().to_owned())
            .count(),
        COUNT,
        "client wallet should have {COUNT} notes before sweeping"
    );

    loop {
        let plans = penumbra_sdk_wallet::plan::sweep(&mut view_client, OsRng)
            .await
            .context("constructing sweep plans")?;
        if plans.is_empty() {
            break;
        }
        for plan in plans {
            let tx = client.witness_auth_build(&plan).await?;
            test_node
                .block()
                .with_data(vec![tx.encode_to_vec()])
                .execute()
                .await?;
        }
    }

    let post_sweep_notes = view_client.unspent_notes_by_address_and_asset().await?;

    client.sync_to_latest(storage.latest_snapshot()).await?;
    assert_eq!(
        post_sweep_notes
            .get(&AddressIndex::from(0))
            .expect("test wallet could not find any notes")
            .get(&*STAKING_TOKEN_ASSET_ID)
            .map(Vec::len),
        Some(2),
        "destination address should have collected {SWEEP_COUNT} notes into one note"
    );

    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
