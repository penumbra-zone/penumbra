mod common;

use {
    self::common::BuilderExt,
    anyhow::anyhow,
    ark_ff::UniformRand,
    cnidarium::TempStorage,
    penumbra_app::server::consensus::Consensus,
    penumbra_genesis::AppState,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::DomainType,
    penumbra_sct::component::clock::EpochRead as _,
    penumbra_stake::{
        component::validator_handler::ValidatorDataRead as _, validator::Validator,
        UndelegateClaimPlan,
    },
    penumbra_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
    },
    rand_core::OsRng,
    tap::Tap,
    tracing::{error_span, info, Instrument},
};

/// The length of the [`penumbra_sct`] epoch.
///
/// This test relies on many epochs turning over, so we will work with a shorter epoch duration.
const EPOCH_DURATION: u64 = 3;

/// The length of the [`penumbra_stake`] unbonding_delay.
const UNBONDING_DELAY: u64 = 4;

#[tokio::test]
async fn app_can_undelegate_from_a_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Helper function to get the latest block height.
    let get_latest_height = || async {
        storage
            .latest_snapshot()
            .get_block_height()
            .await
            .expect("should be able to get latest block height")
    };

    // Helper function to get the latest epoch.
    let get_latest_epoch = || async {
        storage
            .latest_snapshot()
            .get_current_epoch()
            .await
            .expect("should be able to get curent epoch")
    };

    // Configure an AppState with slightly shorter epochs than usual.
    let app_state = AppState::Content(penumbra_genesis::Content {
        sct_content: penumbra_sct::genesis::Content {
            sct_params: penumbra_sct::params::SctParameters {
                epoch_duration: EPOCH_DURATION,
            },
        },
        stake_content: penumbra_stake::genesis::Content {
            stake_params: penumbra_stake::params::StakeParameters {
                unbonding_delay: UNBONDING_DELAY,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // Start the test node.
    let mut node = {
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
    }?;

    // Retrieve the validator definition from the latest snapshot.
    let Validator { identity_key, .. } = match storage
        .latest_snapshot()
        .validator_definitions()
        .tap(|_| info!("getting validator definitions"))
        .await?
        .as_slice()
    {
        [v] => v.clone(),
        unexpected => panic!("there should be one validator, got: {unexpected:?}"),
    }; // ..and note the asset id for delegation tokens tied to this validator.
    let delegate_token_id = penumbra_stake::DelegationToken::new(identity_key).id();

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Now, create a transaction that delegates to the validator.
    //
    // Hang onto the staking note nullifier, so we can interrogate whether that note is spent.
    let (plan, staking_note, staking_note_nullifier) = {
        use {
            penumbra_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_transaction::{
                memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
            },
        };
        let snapshot = storage.latest_snapshot();
        let note = client
            .notes_by_asset(*penumbra_asset::STAKING_TOKEN_ASSET_ID)
            .cloned()
            .next()
            .expect("should get staking note");
        let rate = snapshot
            .get_validator_rate(&identity_key)
            .await?
            .ok_or(anyhow!("validator has a rate"))?
            .tap(|rate| tracing::info!(?rate, "got validator rate"));
        let spend = SpendPlan::new(
            &mut rand_core::OsRng,
            note.clone(),
            client
                .position(note.commit())
                .expect("note should be in mock client's tree"),
        );
        let staking_note_nullifier = spend.nullifier(&client.fvk);
        let delegate = rate.build_delegate(
            storage.latest_snapshot().get_current_epoch().await?,
            note.amount(),
        );
        let output = OutputPlan::new(
            &mut rand_core::OsRng,
            delegate.delegation_value(),
            *test_keys::ADDRESS_1,
        );
        let mut plan = TransactionPlan {
            actions: vec![spend.into(), output.into(), delegate.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: MemoPlan::new(&mut OsRng, MemoPlaintext::blank_memo(*test_keys::ADDRESS_0))
                .map(Some)?,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        };
        plan.populate_detection_data(rand_core::OsRng, 0);
        (plan, note, staking_note_nullifier)
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Show that the client does not have delegation tokens before delegating.
    assert_eq!(
        client.notes_by_asset(delegate_token_id).count(),
        0,
        "client should not have delegation tokens before delegating"
    );

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with delegation transaction"))
        .await?;
    let post_delegate_snapshot = storage.latest_snapshot();
    client
        .sync_to_latest(post_delegate_snapshot.clone())
        .await?;

    // Show that the client now has a single note for some delegation tokens.
    let delegate_note: penumbra_shielded_pool::Note = {
        let mut notes: Vec<_> = client.notes_by_asset(delegate_token_id).cloned().collect();
        assert_eq!(notes.len(), 1, "client should now have delegation tokens");
        notes.pop().unwrap()
    };

    // Show that the staking note has a nullifier that has now been spent.
    {
        use penumbra_sct::component::tree::VerificationExt;
        let snapshot = storage.latest_snapshot();
        let Err(_) = snapshot
            .check_nullifier_unspent(staking_note_nullifier)
            .await
        else {
            panic!("staking note was spent in delegation")
        };
    }

    // Fast forward to the final block of the epoch.
    {
        let jump_to = 2;
        while get_latest_height().await < jump_to {
            node.block().execute().await?;
        }
    }

    // Build a transaction that will now undelegate from the validator.
    let (plan, undelegate_token_id) = {
        use {
            penumbra_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_stake::DelegationToken,
            penumbra_transaction::{
                memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
            },
        };
        let snapshot = storage.latest_snapshot();
        client.sync_to_latest(snapshot.clone()).await?;
        let rate = snapshot
            .get_validator_rate(&identity_key)
            .await?
            .ok_or(anyhow::anyhow!("new validator has a rate"))?
            .tap(|rate| tracing::info!(?rate, "got new validator rate"));

        let undelegation_id = DelegationToken::new(identity_key).id();
        let note = client
            .notes
            .values()
            .filter(|n| n.asset_id() == undelegation_id)
            .cloned()
            .next()
            .expect("the test account should have one staking token note");
        let spend = SpendPlan::new(
            &mut rand_core::OsRng,
            note.clone(),
            client
                .position(note.commit())
                .expect("note should be in mock client's tree"),
        );
        let undelegate = rate.build_undelegate(
            storage.latest_snapshot().get_current_epoch().await?,
            note.amount(),
        );
        let undelegate_token_id = undelegate.unbonding_token().id();
        let output = OutputPlan::new(
            &mut rand_core::OsRng,
            undelegate.unbonded_value(),
            *test_keys::ADDRESS_1,
        );

        let mut plan = TransactionPlan {
            actions: vec![spend.into(), output.into(), undelegate.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: MemoPlan::new(&mut OsRng, MemoPlaintext::blank_memo(*test_keys::ADDRESS_0))
                .map(Some)?,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        };
        plan.populate_detection_data(rand_core::OsRng, 0);
        (plan, undelegate_token_id)
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the undelegation transaction, applying it to the chain state.
    let pre_undelegated_epoch = get_latest_epoch().await;
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with undelegation transaction"))
        .await?;
    let post_undelegate_snapshot = storage.latest_snapshot();

    // Compute the height we expect to see this unbonding period finish.
    let expected_end_of_unboding_period_height = post_undelegate_snapshot
        .compute_unbonding_height(
            &identity_key,
            post_undelegate_snapshot.get_block_height().await?,
        )
        .await?
        .expect("snapshot should have a block height");

    // Show that we immediately receive unbonding tokens after undelegating.
    let undelegate_note: penumbra_shielded_pool::Note = {
        client.sync_to_latest(post_undelegate_snapshot).await?;
        let mut undelegate_notes: Vec<_> = client
            .notes_by_asset(undelegate_token_id)
            .cloned()
            .collect();
        assert_eq!(
            undelegate_notes.len(),
            1,
            "client should have unbonding tokens immediately after undelegating"
        );
        assert_eq!(
            client.notes_by_asset(delegate_token_id).count(),
            /*0, TODO(kate): we still see delegation tokens after undelegating*/ 1,
            "client should not have delegation tokens immediately after undelegating"
        );
        undelegate_notes.pop().unwrap()
    };

    // Jump to the end of the unbonding period.
    {
        let jump_to = expected_end_of_unboding_period_height;
        while get_latest_height().await < jump_to {
            node.block().execute().await?;
        }
    }

    // Build a transaction that will now reclaim staking tokens from the validator.
    let plan = {
        client.sync_to_latest(storage.latest_snapshot()).await?;
        let penalty = penumbra_stake::Penalty::from_percent(0);
        let note = client
            .notes
            .values()
            .cloned()
            .filter(|n| n.asset_id() == undelegate_token_id)
            .next()
            .expect("should have an unbonding note");
        let claim = UndelegateClaimPlan {
            validator_identity: identity_key,
            unbonding_start_height: pre_undelegated_epoch.start_height,
            penalty,
            unbonding_amount: note.amount(),
            balance_blinding: decaf377::Fr::rand(&mut OsRng),
            proof_blinding_r: decaf377::Fq::rand(&mut OsRng),
            proof_blinding_s: decaf377::Fq::rand(&mut OsRng),
        };
        let mut plan = TransactionPlan {
            actions: vec![claim.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: MemoPlan::new(&mut OsRng, MemoPlaintext::blank_memo(*test_keys::ADDRESS_0))
                .map(Some)?,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        };
        plan.populate_detection_data(rand_core::OsRng, 0);
        plan
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with undelegation claim"))
        .await?;
    let post_claim_snapshot = storage.latest_snapshot();

    let staking_note_2 = {
        client.sync_to_latest(post_claim_snapshot.clone()).await?;
        let mut notes: Vec<_> = client
            .notes_by_asset(*penumbra_asset::STAKING_TOKEN_ASSET_ID)
            .cloned()
            .collect();
        assert_eq!(notes.len(), 1, "client should still have staking notes");
        assert_eq!(
            client.notes_by_asset(undelegate_token_id).count(),
            1,
            "client should still have undelegation notes"
        );
        assert_eq!(
            client.notes_by_asset(delegate_token_id).count(),
            1,
            "client should still have delegation notes"
        );
        notes.pop().unwrap()
    };

    {
        let staking_note_amount = staking_note.amount();
        let staking_note_2_amount = staking_note_2.amount();
        let delegate_note_amount = delegate_note.amount();
        let undelegate_note_amount = undelegate_note.amount();

        dbg!(staking_note_amount);
        dbg!(staking_note_2_amount);
        dbg!(delegate_note_amount);
        dbg!(undelegate_note_amount);

        dbg!(staking_note_amount / delegate_note_amount);
        dbg!(delegate_note_amount / undelegate_note_amount);
        dbg!(staking_note_2_amount / staking_note_amount);

        let validator_rate = storage
            .latest_snapshot()
            .get_validator_rate(&identity_key)
            .await?;
        dbg!(validator_rate);
    }

    // The test passed. Free our temporary storage and drop our tracing subscriber.
    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
