//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

use {
    self::common::BuilderExt,
    anyhow::anyhow,
    cnidarium::TempStorage,
    penumbra_app::{genesis::AppState, server::consensus::Consensus},
    penumbra_mock_consensus::TestNode,
    penumbra_sct::component::clock::EpochRead as _,
    penumbra_stake::component::validator_handler::ValidatorDataRead as _,
    tap::{Tap, TapFallible},
    tracing::info,
};

mod common;

/// Exercises that a test node can be instantiated using the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_an_init_chain_request() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let test_node = {
        let app_state = AppState::default();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // Free our temporary storage.
    drop(test_node);
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
    let test_node = {
        let app_state = AppState::default();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    let snapshot = storage.latest_snapshot();
    let validators = snapshot
        .validator_definitions()
        .tap(|_| info!("getting validator definitions"))
        .await?;
    match validators.as_slice() {
        [v] => {
            let identity_key = v.identity_key;
            let status = snapshot
                .get_validator_state(&identity_key)
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
    drop(test_node);
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
    let mut test_node = {
        let app_state = AppState::default();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    let height = || async { storage.latest_snapshot().get_block_height().await };

    // Fast forward eight blocks, and show that the height is 8 after doing so.
    assert_eq!(height().await?, 0, "height should begin at 0");
    test_node.fast_forward(8).await?;
    assert_eq!(height().await?, 8_u64, "height should grow");

    // Free our temporary storage.
    drop(test_node);
    drop(storage);
    drop(guard);

    Ok(())
}
