//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use {
    anyhow::anyhow, cnidarium::TempStorage, penumbra_sct::component::clock::EpochRead,
    penumbra_stake::component::validator_handler::ValidatorDataRead as _, tap::Tap, tracing::info,
};

/// Exercises that a test node can be instantiated using the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_an_init_chain_request() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let _ = common::start_test_node(&storage).await?;

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}

/// Exercises that the mock consensus engine can provide a single genesis validator.
#[tokio::test]
async fn mock_consensus_can_define_a_genesis_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let _test_node = common::start_test_node(&storage).await?;

    let snapshot = storage.latest_snapshot();
    let validators = snapshot
        .validator_definitions()
        .tap(|_| info!("getting validator definitions"))
        .await?;
    match validators.as_slice() {
        [v] => {
            let status = snapshot
                .get_validator_state(&v.identity_key)
                .await?
                .ok_or_else(|| anyhow!("could not find validator status"))?;
            assert_eq!(
                status,
                penumbra_stake::validator::State::Active,
                "validator should be active"
            );
        }
        unexpected => panic!("there should be one validator, got: {unexpected:?}"),
    }

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}

/// Exercises that a series of empty blocks, with no validator set present, can be successfully
/// executed by the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_a_sequence_of_empty_blocks() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let mut test_node = common::start_test_node(&storage).await?;

    let height = || async { storage.latest_snapshot().get_block_height().await };

    // Fast forward eight blocks, and show that the height is 8 after doing so.
    assert_eq!(height().await?, 0, "height should begin at 0");
    test_node.fast_forward(8).await?;
    assert_eq!(height().await?, 8_u64, "height should grow");

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
