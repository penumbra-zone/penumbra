use {
    self::common::BuilderExt,
    anyhow::anyhow,
    cnidarium::TempStorage,
    penumbra_app::{
        genesis::{AppState, Content},
        server::consensus::Consensus,
    },
    penumbra_community_pool::{CommunityPoolDeposit, StateReadExt},
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::DomainType,
    penumbra_shielded_pool::SpendPlan,
    penumbra_transaction::{TransactionParameters, TransactionPlan},
    rand_core::OsRng,
    tap::{Tap, TapFallible},
    tracing::info,
};

mod common;

/// Exercises that the app can *NOT* deposit a note into the community pool when disabled.
#[tokio::test]
async fn app_cannot_deposit_into_community_pool_when_disabled() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Define our application state, and start the test node.
    let mut test_node = {
        let app_state = AppState::Content(Content {
            governance_content: penumbra_governance::genesis::Content {
                governance_params: penumbra_governance::params::GovernanceParameters {
                    community_pool_is_frozen: true,
                    ..Default::default()
                },
            },
            ..Default::default()
        });
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Take one of the test wallet's notes, and prepare to deposit it in the community pool.
    let note = client
        .notes
        .values()
        .cloned()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;

    // Create a community pool transaction.
    let mut plan = {
        let value = note.value();
        let spend = SpendPlan::new(
            &mut OsRng,
            note.clone(),
            client
                .position(note.commit())
                .ok_or_else(|| anyhow!("input note commitment was unknown to mock client"))?,
        )
        .into();
        let deposit = CommunityPoolDeposit { value }.into();
        TransactionPlan {
            actions: vec![spend, deposit],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: None,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
    };
    plan.populate_detection_data(OsRng, 0);
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    let pre_tx_snapshot = storage.latest_snapshot();
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;
    let post_tx_snapshot = storage.latest_snapshot();

    // Show that nothing happened.
    {
        let pre_tx_balance = pre_tx_snapshot.community_pool_balance().await?;
        let post_tx_balance = post_tx_snapshot.community_pool_balance().await?;
        assert_eq!(
            pre_tx_balance, post_tx_balance,
            "community pool should not have been updated"
        );
    }

    // Free our temporary storage.
    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
