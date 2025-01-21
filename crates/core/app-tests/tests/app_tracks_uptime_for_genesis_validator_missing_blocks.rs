use {
    self::common::{BuilderExt, ValidatorDataReadExt},
    anyhow::Context,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_stake::component::validator_handler::validator_store::ValidatorDataRead,
    tap::Tap,
    tracing::{error_span, trace, Instrument},
};

mod common;

#[tokio::test]
async fn app_tracks_uptime_for_genesis_validator_missing_blocks() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Start the test node.
    let mut node = {
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
    }?;

    // Retrieve the validator definition from the latest snapshot.
    let [identity_key] = storage
        .latest_snapshot()
        .validator_identity_keys()
        .await?
        .try_into()
        .map_err(|keys| anyhow::anyhow!("expected one key, got: {keys:?}"))?;
    let get_uptime = || async {
        storage
            .latest_snapshot()
            .get_validator_uptime(&identity_key)
            .await
            .expect("should be able to get a validator uptime")
            .expect("validator uptime should exist")
    };

    // Show that the uptime starts at with no missed blocks.
    assert_eq!(
        get_uptime().await.num_missed_blocks(),
        0,
        "no blocks have been missed at genesis"
    );

    // Jump ahead a few blocks.
    let height = 4;
    for i in 1..=height {
        node.block()
            .without_signatures()
            .execute()
            .tap(|_| trace!(%i, "executing block with no signatures"))
            .instrument(error_span!("executing block with no signatures", %i))
            .await
            .context("executing block with no signatures")?;
    }

    // Check the validator's uptime once more. We should have uptime data up to the fourth block,
    // and the validator should have missed all of the blocks between genesis and now.
    {
        let uptime = get_uptime().await;
        assert_eq!(uptime.as_of_height(), height);
        assert_eq!(
            uptime.num_missed_blocks(),
            /*NB: this is off-by-one */ (height - 1) as usize,
            "validator should have missed the last {height} blocks"
        );
    }

    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
