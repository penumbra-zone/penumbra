use {
    self::common::BuilderExt,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_sct::component::clock::EpochRead as _,
    tap::TapFallible,
};

mod common;

/// Exercises that a series of empty blocks, with no validator set present, can be successfully
/// executed by the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_a_sequence_of_empty_blocks() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;
    let mut test_node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
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
