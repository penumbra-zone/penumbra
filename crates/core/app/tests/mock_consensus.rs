//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use cnidarium::TempStorage;
use penumbra_app::server::consensus::Consensus;

#[tokio::test]
async fn mock_consensus_can_send_a_failing_init_chain_request() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Instantiate the consensus service, and start the test node.
    use penumbra_mock_consensus::TestNode;
    let consensus = Consensus::new(storage.as_ref().clone());
    let engine = TestNode::builder()
        .single_validator()
        .app_state(() /*genesis::AppState::default()*/)
        .init_chain(consensus)
        .await;

    // NB: we don't expect this to succeed... yet.
    assert!(engine.is_err(), "init_chain does not return an Ok(()) yet");

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
