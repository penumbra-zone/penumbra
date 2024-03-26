mod common;

use {
    anyhow::anyhow,
    cnidarium::TempStorage,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::DomainType,
    penumbra_sct::component::tree::SctRead as _,
    penumbra_shielded_pool::{OutputPlan, SpendPlan},
    penumbra_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
    },
    rand_core::OsRng,
    tap::Tap,
    tracing::info,
};

#[tokio::test]
async fn app_can_spend_notes_and_detect_outputs() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let mut test_node = common::start_test_node(&storage).await?;

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Take one of the test wallet's notes, and send it to a different account.
    let input_note = client
        .notes
        .values()
        .cloned()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;

    // Write down a transaction plan with exactly one spend and one output.
    let mut plan = TransactionPlan {
        actions: vec![
            // First, spend the selected input note.
            SpendPlan::new(
                &mut OsRng,
                input_note.clone(),
                // Spends require _positioned_ notes, in order to compute their nullifiers.
                client
                    .position(input_note.commit())
                    .ok_or_else(|| anyhow!("input note commitment was unknown to mock client"))?,
            )
            .into(),
            // Next, create a new output of the exact same amount.
            OutputPlan::new(&mut OsRng, input_note.value(), *test_keys::ADDRESS_1).into(),
        ],
        // Now fill out the remaining parts of the transaction needed for verification:
        memo: Some(MemoPlan::new(
            &mut OsRng,
            MemoPlaintext::blank_memo(*test_keys::ADDRESS_0),
        )?),
        detection_data: None, // We'll set this automatically below
        transaction_parameters: TransactionParameters {
            chain_id: TestNode::<()>::CHAIN_ID.to_string(),
            ..Default::default()
        },
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

    // Check that the nullifiers were spent as a result of the transaction:
    for nf in tx.spent_nullifiers() {
        assert!(pre_tx_snapshot.spend_info(nf).await?.is_none());
        assert!(post_tx_snapshot.spend_info(nf).await?.is_some());
    }

    // Sync the client up to the current block
    client.sync_to_latest(post_tx_snapshot).await?;

    // Check that the client was able to detect the new note:

    // Grab the output note we're expecting to see...
    let output_nc = tx
        .outputs()
        .next()
        .expect("tx has one output")
        .body
        .note_payload
        .note_commitment
        .clone();
    // ... and check that it's now in the client's note set.
    assert!(client.notes.contains_key(&output_nc));

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
