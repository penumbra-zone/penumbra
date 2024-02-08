//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

use {cnidarium::TempStorage, penumbra_app::app::App, penumbra_mock_consensus::TestNode};

#[tokio::test]
#[should_panic]
async fn an_app_with_mock_consensus_can_be_instantiated() {
    let storage = TempStorage::new().await.unwrap();
    let app = App::new(storage.latest_snapshot());
    let _engine = TestNode::<()>::builder()
        .single_validator()
        .app_state(() /*genesis::AppState::default()*/)
        .init_chain(app)
        .await;
}
