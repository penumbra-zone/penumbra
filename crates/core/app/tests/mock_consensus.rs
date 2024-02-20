//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use {
    anyhow::anyhow,
    cnidarium::TempStorage,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_sct::component::clock::EpochRead,
    tracing::{error_span, Instrument},
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
    let mut rng = <rand_chacha::ChaChaRng as rand_core::SeedableRng>::seed_from_u64(0xBEEF);

    // Sync the mock client, using the test account's full viewing key, to the latest snapshot.
    let MockClient { notes, sct, .. } = MockClient::new(test_keys::FULL_VIEWING_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?;

    // Take one of the test account's notes...
    let note = notes
        .values()
        .cloned()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;
    let asset_id = note.asset_id();
    let proof = sct
        .witness(note.commit())
        .ok_or_else(|| anyhow!("index is not witnessed"))?;

    // ...and use it to craft a `Spend`.
    let (spend, spend_key) = {
        use {decaf377_rdsa::SigningKey, penumbra_shielded_pool::SpendPlan};
        let spend_plan = SpendPlan::new(&mut rng, note, proof.position());
        let auth_sig = test_keys::SPEND_KEY
            .spend_auth_key()
            .randomize(&spend_plan.randomizer)
            .sign(&mut rng, [0u8; 64].as_ref());
        let spend = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof, sct.root());
        let key = SigningKey::from(spend_plan.value_blinding);
        (spend, key)
    };

    // Next, craft a transaction, containing this `Spend`.
    let tx = {
        use {
            penumbra_asset::Value,
            penumbra_fee::Fee,
            penumbra_num::Amount,
            penumbra_transaction::{Action, Transaction, TransactionBody, TransactionParameters},
            penumbra_txhash::AuthorizingData,
        };
        let transaction_parameters = TransactionParameters {
            expiry_height: 0,
            chain_id: "i-wonder-if-this-is-load-bearing".to_owned(),
            fee: Fee(Value {
                amount: Amount::zero(),
                asset_id,
            }),
        };
        let transaction_body = TransactionBody {
            actions: vec![Action::Spend(spend)],
            transaction_parameters,
            ..Default::default()
        };
        let binding_sig = spend_key.sign(rng, transaction_body.auth_hash().as_bytes());
        let transaction = Transaction {
            transaction_body,
            binding_sig,
            anchor: sct.root(),
        };
        <Transaction as penumbra_proto::DomainType>::encode_to_vec(&transaction)
    };

    // Execute the transaction, and sync another mock client up to the latest snapshot.
    test_node.block().with_data(vec![tx]).execute().await?;
    MockClient::new(test_keys::FULL_VIEWING_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?;

    // Free our temporary storage.
    drop(storage);
    drop(guard);

    Ok(())
}
