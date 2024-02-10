//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

use {cnidarium::TempStorage, penumbra_mock_consensus::TestNode};

#[tokio::test]
#[should_panic]
#[allow(unreachable_code, unused)]
async fn an_app_with_mock_consensus_can_be_instantiated() {
    let storage = TempStorage::new().await.unwrap();

    // TODO(kate): bind this to an in-memory channel/writer instead.
    let addr: std::net::SocketAddr = todo!();
    let abci_server = penumbra_app::server::new(todo!()).listen_tcp(addr);

    let _engine = TestNode::<()>::builder()
        .single_validator()
        .app_state(() /*genesis::AppState::default()*/)
        .init_chain(abci_server)
        .await;
}
