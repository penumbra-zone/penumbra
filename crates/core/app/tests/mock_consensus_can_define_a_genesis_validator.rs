use {
    self::common::{BuilderExt, ValidatorDataReadExt},
    anyhow::anyhow,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_stake::component::validator_handler::ValidatorDataRead as _,
    tap::{Tap, TapFallible},
    tracing::info,
};

mod common;

/// Exercises that the mock consensus engine can provide a single genesis validator.
#[tokio::test]
async fn mock_consensus_can_define_a_genesis_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;
    let test_node = {
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
                penumbra_sdk_stake::validator::State::Active,
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
