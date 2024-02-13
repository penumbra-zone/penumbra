//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use cnidarium::TempStorage;
use penumbra_app::server::consensus::Consensus;
use penumbra_genesis::AppState;

#[tokio::test]
async fn mock_consensus_can_send_an_init_chain_request() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Instantiate the consensus service, and start the test node.
    let engine = {
        use penumbra_mock_consensus::TestNode;
        let app_state = AppState::default();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .app_state(app_state)
            .init_chain(consensus)
            .await?
    };

    tracing::info!(hash = %engine.last_app_hash_hex(), "finished init chain");

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
