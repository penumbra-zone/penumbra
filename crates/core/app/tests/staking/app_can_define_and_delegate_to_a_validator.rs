#[path = "../common/mod.rs"]
mod common;

use {
    self::common::BuilderExt,
    anyhow::{anyhow, Context},
    cnidarium::TempStorage,
    decaf377_rdsa::{SigningKey, SpendAuth},
    penumbra_app::server::consensus::Consensus,
    penumbra_genesis::AppState,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::DomainType,
    penumbra_stake::{
        component::validator_handler::ValidatorDataRead as _, validator::Validator, FundingStreams,
        GovernanceKey, IdentityKey,
    },
    rand_core::OsRng,
    tap::Tap,
    tracing::{error_span, info, Instrument},
};

/// The length of the [`penumbra_sct`] epoch.
///
/// This test relies on many epochs turning over, so we will work with a shorter epoch duration.
const EPOCH_DURATION: u64 = 8;

#[tokio::test]
async fn app_can_define_and_delegate_to_a_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Configure an AppState with slightly shorter epochs than usual.
    let app_state = AppState::Content(penumbra_genesis::Content {
        sct_content: penumbra_sct::genesis::Content {
            sct_params: penumbra_sct::params::SctParameters {
                epoch_duration: EPOCH_DURATION,
            },
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

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // Fast forward to the next epoch.
    let snapshot_start = storage.latest_snapshot();
    node.fast_forward(EPOCH_DURATION)
        .instrument(error_span!("fast forwarding test node to second epoch"))
        .await
        .context("fast forwarding {EPOCH_LENGTH} blocks")?;
    let snapshot_end = storage.latest_snapshot();

    // Retrieve the validator definition from the latest snapshot.
    let existing_validator = match snapshot_end
        .validator_definitions()
        .tap(|_| info!("getting validator definitions"))
        .await?
        .as_slice()
    {
        [v] => v.clone(),
        unexpected => panic!("there should be one validator, got: {unexpected:?}"),
    };

    let existing_validator_id = existing_validator.identity_key;

    // Check that we are now in a new epoch.
    {
        use penumbra_sct::{component::clock::EpochRead, epoch::Epoch};
        let Epoch { index: start, .. } = snapshot_start.get_current_epoch().await?;
        let Epoch { index: end, .. } = snapshot_end.get_current_epoch().await?;
        assert_eq!(start, 0, "we should start in the first epoch");
        assert_eq!(end, 1, "we should now be in the second epoch");
    }

    // Show that the existing validator is and was active.
    {
        use penumbra_stake::{component::validator_handler::ValidatorDataRead, validator::State};
        let start = snapshot_start
            .get_validator_state(&existing_validator_id)
            .await?;
        let end = snapshot_end
            .get_validator_state(&existing_validator_id)
            .await?;
        assert_eq!(start, Some(State::Active));
        assert_eq!(end, Some(State::Active));
    }

    // Show that the validator was, and still is, in the consensus set.
    {
        use penumbra_stake::component::ConsensusIndexRead;
        let start = snapshot_start.get_consensus_set().await?;
        let end = snapshot_end.get_consensus_set().await?;
        let expected = [existing_validator_id];
        assert_eq!(
            start, expected,
            "validator should start in the consensus set"
        );
        assert_eq!(end, expected, "validator should stay in the consensus set");
    }

    // To define a validator, we need to define two keypairs: an identity key
    // for the Penumbra application and a consensus key for cometbft.
    let new_validator_id_sk = SigningKey::<SpendAuth>::new(OsRng);
    let new_validator_id = IdentityKey(new_validator_id_sk.into());
    let new_validator_consensus_sk = ed25519_consensus::SigningKey::new(OsRng);
    let new_validator_consensus = new_validator_consensus_sk.verification_key();

    // Insert the validator's consensus keypair into the keyring so it can be used to sign blocks.
    node.keyring_mut()
        // Keyring should just be a BTreeMap rather than creating a new API
        .insert(new_validator_consensus, new_validator_consensus_sk);

    // Now define the validator's configuration data.
    let new_validator = Validator {
        identity_key: new_validator_id.clone(),
        // TODO: when https://github.com/informalsystems/tendermint-rs/pull/1401 is released,
        // replace this with a direct `Into::into()` call. at the time of writing, v0.35.0 is the
        // latest version. check for new releases at https://crates.io/crates/tendermint/versions.
        consensus_key: tendermint::PublicKey::from_raw_ed25519(&new_validator_consensus.to_bytes())
            .expect("consensus key is valid"),
        governance_key: GovernanceKey(new_validator_id_sk.into()),
        enabled: true,
        sequence_number: 0,
        name: "test validator".to_string(),
        website: String::default(),
        description: String::default(),
        funding_streams: FundingStreams::default(),
    };

    // Make a transaction that defines a new validator.
    let plan = {
        use {
            penumbra_stake::validator,
            penumbra_transaction::{ActionPlan, TransactionParameters, TransactionPlan},
            rand_core::OsRng,
        };
        let bytes = new_validator.encode_to_vec();
        let auth_sig = new_validator_id_sk.sign(OsRng, &bytes);
        let action = ActionPlan::ValidatorDefinition(validator::Definition {
            validator: new_validator.clone(),
            auth_sig,
        });
        let mut plan = TransactionPlan {
            actions: vec![action.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: None,
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
        .instrument(error_span!(
            "executing block with validator definition transaction"
        ))
        .await?;
    let post_tx_snapshot = storage.latest_snapshot();

    // Show that the set of validators looks correct.
    {
        use penumbra_stake::{component::ConsensusIndexRead, validator::State};
        let snapshot = post_tx_snapshot.clone();
        info!("checking consensus set in block after validator definition");
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&existing_validator_id).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should be defined, but not yet active. It should not be inclueded in
        // consensus yet.
        assert_eq!(
            snapshot.get_validator_state(&new_validator_id).await?,
            Some(State::Defined),
            "new validator definition should be defined but not active"
        );
        // The original validator should still be the only validator in the consensus set.
        assert_eq!(
            snapshot_start.get_consensus_set().await?.len(),
            1,
            "the new validator should not be part of the consensus set yet"
        );
    }

    // Show that the validators have different rates. The second validator was created later, and
    // should thus have a different rate than a validator that has existed since genesis.
    {
        use penumbra_stake::component::validator_handler::validator_store::ValidatorDataRead;
        let snapshot = post_tx_snapshot;
        let existing = snapshot
            .get_validator_rate(&existing_validator_id)
            .await?
            .ok_or(anyhow!("existing validator has a rate"))?;
        let new = snapshot
            .get_validator_rate(&new_validator_id)
            .await?
            .ok_or(anyhow!("new validator has a rate"))?;
        assert_ne!(
            existing, new,
            "validators created at different times should have different rates"
        );
    }

    // Now, create a transaction that delegates to the new validator.
    let plan = {
        use {
            penumbra_asset::STAKING_TOKEN_ASSET_ID,
            penumbra_sct::component::clock::EpochRead,
            penumbra_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_transaction::{
                memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
            },
        };
        let snapshot = storage.latest_snapshot();
        client.sync_to_latest(snapshot.clone()).await?;
        let rate = snapshot
            .get_validator_rate(&new_validator_id)
            .await?
            .ok_or(anyhow!("new validator has a rate"))?
            .tap(|rate| tracing::info!(?rate, "got new validator rate"));
        let note = client
            .notes
            .values()
            .filter(|n| n.asset_id() == *STAKING_TOKEN_ASSET_ID)
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
        plan
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with delegation transaction"))
        .await?;
    let post_delegate_snapshot = storage.latest_snapshot();

    // Show that the set of validators still looks correct. We should not see any changes yet.
    {
        use penumbra_stake::{component::ConsensusIndexRead, validator::State};
        let snapshot = post_delegate_snapshot;
        info!("checking consensus set in block after delegation");
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&existing_validator_id).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should be defined, but not yet active. It should not be inclueded in
        // consensus yet.
        assert_eq!(
            snapshot.get_validator_state(&new_validator_id).await?,
            Some(State::Defined),
            "new validator definition should be defined but not active"
        );
        // The original validator should still be the only validator in the consensus set.
        assert_eq!(
            snapshot.get_consensus_set().await?.len(),
            1,
            "the new validator should not be part of the consensus set yet"
        );
    }

    // Fast forward to the next epoch.
    node.fast_forward(EPOCH_DURATION)
        .instrument(error_span!(
            "fast forwarding test node to epoch after delegation"
        ))
        .await
        .context("fast forwarding {EPOCH_LENGTH} blocks")?;
    let post_delegate_next_epoch_snapshot = storage.latest_snapshot();

    // Show that now, after an epoch and with a delegation, the validator is marked active.
    {
        use penumbra_stake::{component::ConsensusIndexRead, validator::State};
        info!("checking consensus set in epoch after delegation");
        let snapshot = post_delegate_next_epoch_snapshot;
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&existing_validator_id).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should now be active.
        assert_eq!(
            snapshot.get_validator_state(&new_validator_id).await?,
            Some(State::Active),
            "new validator should be active"
        );
        // There should now be two validators in the consensus set.
        assert_eq!(
            snapshot.get_consensus_set().await?.len(),
            2,
            "the new validator should now be part of the consensus set"
        );
    }

    // Build a transaction that will now undelegate from the validator.
    let plan = {
        use {
            penumbra_sct::component::clock::EpochRead,
            penumbra_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_stake::DelegationToken,
            penumbra_transaction::{
                memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
            },
        };
        let snapshot = storage.latest_snapshot();
        client.sync_to_latest(snapshot.clone()).await?;
        let rate = snapshot
            .get_validator_rate(&new_validator_id)
            .await?
            .ok_or(anyhow!("new validator has a rate"))?
            .tap(|rate| tracing::info!(?rate, "got new validator rate"));

        let undelegation_id = DelegationToken::new(new_validator_id).id();
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
        plan
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with undelegation transaction"))
        .await?;
    let post_undelegate_snapshot = storage.latest_snapshot();

    // Show that the consensus set has not changed yet.
    {
        use penumbra_stake::{component::ConsensusIndexRead, validator::State};
        let snapshot = post_undelegate_snapshot;
        info!("checking consensus set in block after undelegation");
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&existing_validator_id).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should now be active.
        assert_eq!(
            snapshot.get_validator_state(&new_validator_id).await?,
            Some(State::Active),
            "new validator should be active"
        );
        // There should now be two validators in the consensus set.
        assert_eq!(
            snapshot.get_consensus_set().await?.len(),
            2,
            "the new validator should now be part of the consensus set"
        );
    }

    // Fast forward to the next epoch.
    node.fast_forward(EPOCH_DURATION)
        .instrument(error_span!(
            "fast forwarding test node to epoch after undelegation"
        ))
        .await
        .context("fast forwarding {EPOCH_LENGTH} blocks")?;
    let post_undelegate_next_epoch_snapshot = storage.latest_snapshot();

    // Show that after undelegating, the validator is no longer marked active.
    {
        use penumbra_stake::{component::ConsensusIndexRead, validator::State};
        info!("checking consensus set in epoch after undelegation");
        let snapshot = post_undelegate_next_epoch_snapshot;
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&existing_validator_id).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should now have reverted to be defined. It no longer has enough
        // delegated stake to participate in consensus.
        assert_eq!(
            snapshot.get_validator_state(&new_validator_id).await?,
            Some(State::Defined),
            "new validator definition should be defined but not active"
        );
        assert_eq!(
            snapshot.get_consensus_set().await?.len(),
            1,
            "the new validator should not be part of the consensus set yet"
        );
    }

    // The test passed. Free our temporary storage and drop our tracing subscriber.
    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
