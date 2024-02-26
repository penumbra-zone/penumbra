//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use {
    anyhow::anyhow,
    cnidarium::TempStorage,
    penumbra_fee::Fee,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::DomainType,
    penumbra_sct::component::clock::EpochRead,
    penumbra_shielded_pool::{OutputPlan, SpendPlan},
    penumbra_transaction::{
        memo::MemoPlaintext,
        plan::{CluePlan, DetectionDataPlan, MemoPlan},
        ActionPlan, Transaction, TransactionParameters, TransactionPlan, WitnessData,
    },
    rand_core::OsRng,
    std::ops::Deref,
    tap::Tap,
    tracing::{error_span, info, Instrument},
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

/// Exercises that a series of empty blocks, with no validator set present, can be successfully
/// executed by the consensus service.
#[tokio::test]
async fn mock_consensus_can_send_a_sequence_of_empty_blocks() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let mut test_node = common::start_test_node(&storage).await?;

    // Check that we begin at height 0, before any blocks have been generated.
    assert_eq!(
        storage.latest_snapshot().get_block_height().await?,
        0,
        "height should begin at 0"
    );

    for expected in 1..=8 {
        // Generate an empty block.
        test_node
            .block()
            .with_data(vec![])
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

#[tokio::test]
async fn mock_consensus_can_send_a_spend_action() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;
    let mut test_node = common::start_test_node(&storage).await?;

    // Sync the mock client, using the test account's full viewing key, to the latest snapshot.
    let (viewing_key, spend_key) = (&test_keys::FULL_VIEWING_KEY, &test_keys::SPEND_KEY);
    let client = MockClient::new(viewing_key.deref().clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Take one of the test account's notes...
    let (commitment, note) = client
        .notes
        .iter()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?
        .tap(|(commitment, note)| info!(?commitment, ?note, "mock client note commitment"));

    let spend: ActionPlan = {
        let proof = client
            .sct
            .witness(commitment.clone())
            .ok_or_else(|| anyhow!("commitment is not witnessed"))?;
        let note = note.clone();
        let position = proof.position();
        SpendPlan::new(&mut OsRng, note, position).into()
    };

    // Build a transaction spending this note.
    let tx: Transaction = {
        let chain_id = TestNode::<()>::CHAIN_ID.to_string();
        let transaction_parameters = TransactionParameters {
            expiry_height: 0,
            fee: Fee::default(),
            chain_id,
        };
        let detection_data = Some(DetectionDataPlan {
            clue_plans: vec![CluePlan::new(&mut OsRng, *test_keys::ADDRESS_0, 0)],
        });
        let plan = TransactionPlan {
            actions: vec![
                spend,
                OutputPlan::new(&mut OsRng, note.value(), *test_keys::ADDRESS_1).into(),
            ],
            transaction_parameters,
            detection_data,
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(*test_keys::ADDRESS_1),
            )?),
        };
        let witness = WitnessData {
            anchor: client.sct.root(),
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    (
                        spend.note.commit(),
                        client.sct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let auth = plan.authorize(OsRng, spend_key)?;
        plan.build_concurrent(viewing_key, &witness, &auth).await?
    };

    // Execute the transaction, and sync another mock client up to the latest snapshot.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;

    // Sync to the latest storage snapshot once more.
    let client_after_spend = MockClient::new(viewing_key.deref().clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Show that we performed the spend as expected.
    assert_eq!(
        client_after_spend.notes.len(),
        client.notes.len() + 1,
        "a new note should exist after performing the spend",
    );

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
