use {
    self::common::BuilderExt,
    anyhow::Context,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID,
    penumbra_sdk_keys::{keys::AddressIndex, test_keys},
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::{
        view::v1::{
            view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
            GasPricesRequest, StatusRequest, StatusResponse,
        },
        DomainType,
    },
    penumbra_sdk_view::{Planner, SpendableNoteRecord, ViewClient},
    std::ops::Deref,
    tap::{Tap, TapFallible},
};

mod common;

// NB: a multi-thread runtime is needed to run both the view server and its client.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Flaked in #4627,#4632,#4624"]
async fn view_server_can_be_served_on_localhost() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Instantiate a mock tendermint proxy, which we will connect to the test node.
    let proxy = penumbra_sdk_mock_tendermint_proxy::TestNodeProxy::new::<Consensus>();

    // Start the test node.
    let mut test_node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
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

    let grpc_url = "http://127.0.0.1:8080" // see #4517
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

    let notes = view_client.unspent_notes_by_address_and_asset().await?;
    let staking_notes = notes
        .get(&AddressIndex::default())
        .expect("test wallet could not find any notes")
        .get(&*STAKING_TOKEN_ASSET_ID)
        .expect("test wallet did not contain any staking tokens");

    // Get one of the notes, which we will spend.
    let SpendableNoteRecord { note, position, .. } = staking_notes[0].to_owned();

    // Create a plan spending that note, using the `Planner`.
    let plan = {
        let gas_prices = view_client
            .gas_prices(GasPricesRequest {})
            .await?
            .into_inner()
            .gas_prices
            .expect("gas prices must be available")
            .try_into()?;

        let mut planner = Planner::new(rand_core::OsRng);
        planner
            .set_gas_prices(gas_prices)
            .spend(note.to_owned(), position)
            .output(note.value(), test_keys::ADDRESS_1.deref().clone())
            .plan(&mut view_client, AddressIndex::default())
            .await?
    };
    client.sync_to_latest(storage.latest_snapshot()).await?;
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    let pre_tx_snapshot = storage.latest_snapshot();
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;
    let post_tx_snapshot = storage.latest_snapshot();

    // Check that the nullifiers were spent as a result of the transaction:
    for nf in tx.spent_nullifiers() {
        use penumbra_sdk_sct::component::tree::SctRead as _;
        assert!(pre_tx_snapshot.spend_info(nf).await?.is_none());
        assert!(post_tx_snapshot.spend_info(nf).await?.is_some());
    }

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
                full_sync_height: 11,
                partial_sync_height: 11,
                catching_up: false,
            }
        );
    }

    // Check that the note has been spent, and that a note has appeared in the other address.
    let post_tx_notes = view_client.unspent_notes_by_address_and_asset().await?;
    assert!(
        post_tx_notes
            .get(&AddressIndex::default())
            .expect("test wallet could not find any notes")
            .get(&*STAKING_TOKEN_ASSET_ID)
            .is_none(),
        "saurce address should not be associated with any staking tokens after tx"
    );
    assert_eq!(
        post_tx_notes
            .get(&AddressIndex::from(1))
            .expect("test wallet could not find any notes")
            .get(&*STAKING_TOKEN_ASSET_ID)
            .map(Vec::len),
        Some(1),
        "destination address should have a staking token note after tx"
    );

    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
