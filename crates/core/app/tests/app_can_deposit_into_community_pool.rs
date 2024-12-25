use {
    self::common::BuilderExt,
    anyhow::anyhow,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_asset::asset,
    penumbra_sdk_community_pool::{CommunityPoolDeposit, StateReadExt},
    penumbra_sdk_keys::test_keys,
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_num::Amount,
    penumbra_sdk_proto::DomainType,
    penumbra_sdk_shielded_pool::SpendPlan,
    penumbra_sdk_transaction::{TransactionParameters, TransactionPlan},
    rand_core::OsRng,
    std::collections::BTreeMap,
    tap::{Tap, TapFallible},
    tracing::info,
};

mod common;

/// Exercises that the app can deposit a note into the community pool.
#[tokio::test]
async fn app_can_deposit_into_community_pool() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Define our application state, and start the test node.
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
    plan.populate_detection_data(OsRng, Default::default());
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    let pre_tx_snapshot = storage.latest_snapshot();
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;
    let post_tx_snapshot = storage.latest_snapshot();

    // Assert that the community pool balance looks correct for the deposited asset id, and that
    // other amounts were not affected by the deposit.
    {
        type Balance = BTreeMap<asset::Id, Amount>;

        let id = note.asset_id();
        let pre_tx_balance = pre_tx_snapshot.community_pool_balance().await?;
        let post_tx_balance = post_tx_snapshot.community_pool_balance().await?;

        let get_balance_for_id = |balance: &Balance| balance.get(&id).copied().unwrap_or_default();
        let pre_tx_amount = get_balance_for_id(&pre_tx_balance);
        let post_tx_amount = get_balance_for_id(&post_tx_balance);
        assert_eq!(
            pre_tx_amount + note.amount(),
            post_tx_amount,
            "community pool balance should include the deposited note"
        );

        let count_other_assets_in_pool = |balance: &Balance| {
            balance
                .into_iter()
                // Skip the amount for our note's asset id.
                .filter(|(&entry_id, _)| entry_id != id)
                .map(|(_, &amount)| amount)
                .sum::<Amount>()
        };
        assert_eq!(
            count_other_assets_in_pool(&pre_tx_balance),
            count_other_assets_in_pool(&post_tx_balance),
            "other community pool balance amounts should not have changed"
        );
    }

    // Free our temporary storage.
    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
