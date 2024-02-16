//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use {
    self::common::BuilderExt,
    cnidarium::TempStorage,
    penumbra_app::server::consensus::Consensus,
    penumbra_genesis::AppState,
    penumbra_sct::component::clock::EpochRead,
    tendermint::evidence::List,
    tracing::{error_span, Instrument},
};

/// Exercises that a test node can be instantiated using the consensus service.
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
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await?
    };

    tracing::info!(hash = %engine.last_app_hash_hex(), "finished init chain");

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}

/// Exercises that a series of empty blocks, with no validator set present, can be successfully
/// executed by the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_a_sequence_of_empty_blocks() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Instantiate the consensus service, and start the test node.
    let mut engine = {
        use penumbra_mock_consensus::TestNode;
        let app_state = AppState::default();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await?
    };

    // Check that we begin at height 0, before any blocks have been generated.
    assert_eq!(
        storage.latest_snapshot().get_block_height().await?,
        0,
        "height should begin at 0"
    );

    for expected in 1..=8 {
        // Generate an empty block.
        engine
            .block()
            .with_data(vec![])
            .with_evidence(List::new(Vec::new()))
            .execute()
            .instrument(error_span!("executing block", %expected))
            .await?;

        // Check that the latest snapshot has the expected block height.
        let height = storage.latest_snapshot().get_block_height().await?;
        assert_eq!(
            height, expected,
            "height should continue to incrementally grow"
        );
    }

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
