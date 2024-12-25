use {
    self::common::BuilderExt,
    cnidarium::TempStorage,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_mock_consensus::TestNode,
    std::time::Duration,
    tap::{Tap, TapFallible},
};

mod common;

/// This is more of a test of test code; this ensures
/// that the mock tendermint's block times are set and increment as expected.
#[tokio::test]
async fn mock_tendermint_block_times_correct() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Fixed start time:
    let start_time = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    // Define our application state, and start the test node.
    let mut test_node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
        let consensus = Consensus::new(storage.as_ref().clone());
        // This should use the default time callback of 5s
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .with_initial_timestamp(start_time)
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // The test node's time should be the initial timestamp before any blocks are committed
    assert_eq!(*test_node.timestamp(), start_time);

    // Test a handful of block executions
    for i in 0..10 {
        // Execute a block on the test node
        test_node.block().execute().await?;

        // Ensure the time has incremented by 5 seconds
        assert_eq!(
            *test_node.timestamp(),
            start_time
                .checked_add(Duration::from_secs(5 * (i + 1)))
                .unwrap()
        );
    }

    // Now do it with a different duration.
    let block_duration = Duration::from_secs(13);
    let storage = TempStorage::new().await?;
    let mut test_node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
        let consensus = Consensus::new(storage.as_ref().clone());
        // This should use the default time callback of 5s
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .with_initial_timestamp(start_time)
            // Set a callback to add 13 seconds instead
            .ts_callback(move |t| t.checked_add(block_duration).unwrap())
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // The test node's time should be the initial timestamp before any blocks are committed
    assert_eq!(*test_node.timestamp(), start_time);

    // Test a handful of block executions
    for i in 0..10 {
        // Execute a block on the test node
        test_node.block().execute().await?;

        // Ensure the time has incremented by 5 seconds
        assert_eq!(
            *test_node.timestamp(),
            start_time.checked_add(block_duration * (i + 1)).unwrap()
        );
    }

    // Free our temporary storage.
    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
