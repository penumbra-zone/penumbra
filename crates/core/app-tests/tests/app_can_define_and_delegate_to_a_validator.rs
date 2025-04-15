use {
    self::common::{BuilderExt, TestNodeExt, ValidatorDataReadExt},
    anyhow::anyhow,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey},
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_keys::test_keys,
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::DomainType,
    penumbra_sdk_stake::{
        component::validator_handler::ValidatorDataRead as _, validator::Validator, FundingStreams,
        GovernanceKey, IdentityKey,
    },
    rand_core::OsRng,
    std::ops::Deref,
    tap::Tap,
    tracing::{error_span, info, Instrument},
};

mod common;

/// The length of the [`penumbra_sdk_sct`] epoch.
///
/// This test relies on many epochs turning over, so we will work with a shorter epoch duration.
const EPOCH_DURATION: u64 = 8;

#[tokio::test]
async fn app_can_define_and_delegate_to_a_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Configure an AppState with slightly shorter epochs than usual.
    let app_state = AppState::Content(
        genesis::Content::default()
            .with_epoch_duration(EPOCH_DURATION)
            .with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
    );

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
    node.fast_forward_to_next_epoch(&storage).await?;
    let snapshot_end = storage.latest_snapshot();

    // Retrieve the validator definition from the latest snapshot.
    let [existing_validator_id] = storage
        .latest_snapshot()
        .validator_identity_keys()
        .await?
        .try_into()
        .map_err(|keys| anyhow::anyhow!("expected one key, got: {keys:?}"))?;

    // Check that we are now in a new epoch.
    {
        use penumbra_sdk_sct::{component::clock::EpochRead, epoch::Epoch};
        let Epoch { index: start, .. } = snapshot_start.get_current_epoch().await?;
        let Epoch { index: end, .. } = snapshot_end.get_current_epoch().await?;
        assert_eq!(start, 0, "we should start in the first epoch");
        assert_eq!(end, 1, "we should now be in the second epoch");
    }

    // Show that the existing validator is and was active.
    {
        use penumbra_sdk_stake::{
            component::validator_handler::ValidatorDataRead, validator::State,
        };
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
        use penumbra_sdk_stake::component::ConsensusIndexRead;
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
    let new_validator_id = IdentityKey(VerificationKey::from(&new_validator_id_sk).into());
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
            penumbra_sdk_stake::validator,
            penumbra_sdk_transaction::{ActionPlan, TransactionParameters, TransactionPlan},
            rand_core::OsRng,
        };
        let bytes = new_validator.encode_to_vec();
        let auth_sig = new_validator_id_sk.sign(OsRng, &bytes);
        let action = ActionPlan::ValidatorDefinition(validator::Definition {
            validator: new_validator.clone(),
            auth_sig,
        });
        TransactionPlan {
            actions: vec![action.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: None,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
        .with_populated_detection_data(OsRng, Default::default())
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
        use penumbra_sdk_stake::{component::ConsensusIndexRead, validator::State};
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
        use penumbra_sdk_stake::component::validator_handler::validator_store::ValidatorDataRead;
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
            penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID,
            penumbra_sdk_sct::component::clock::EpochRead,
            penumbra_sdk_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_sdk_transaction::{
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
            test_keys::ADDRESS_1.deref().clone(),
        );
        TransactionPlan {
            actions: vec![spend.into(), output.into(), delegate.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(test_keys::ADDRESS_0.deref().clone()),
            )),
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
        .with_populated_detection_data(OsRng, Default::default())
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    let pre_delegate_snapshot = storage.latest_snapshot();
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with delegation transaction"))
        .await?;
    let post_delegate_snapshot = storage.latest_snapshot();

    // Show that the set of validators still looks correct. We should not see any changes yet.
    {
        use penumbra_sdk_stake::{component::ConsensusIndexRead, validator::State};
        let snapshot = post_delegate_snapshot.clone();
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

    // Confirm that the new validator's voting power has not changed immediately.
    let new_validator_original_power = {
        use penumbra_sdk_sct::component::clock::EpochRead;
        let pre_delegate_power = pre_delegate_snapshot
            .get_validator_power(&new_validator_id)
            .await?
            .expect("should have voting power before delegating");
        let post_delegate_power = post_delegate_snapshot
            .get_validator_power(&new_validator_id)
            .await?
            .expect("should have voting power after delegating");
        debug_assert_eq!(
            pre_delegate_snapshot.get_current_epoch().await?,
            post_delegate_snapshot.get_current_epoch().await?,
            "avoid puzzling errors by confirming that pre- and post-delegation snapshots do not \
             sit upon an epoch boundary"
        );
        assert_eq!(
            pre_delegate_power, post_delegate_power,
            "a delegated validator"
        );
        pre_delegate_power
    };

    // Fast forward to the next epoch.
    node.fast_forward_to_next_epoch(&storage).await?;
    let post_delegate_next_epoch_snapshot = storage.latest_snapshot();

    // Show that now, after an epoch and with a delegation, the validator is marked active.
    {
        use penumbra_sdk_stake::{component::ConsensusIndexRead, validator::State};
        info!("checking consensus set in epoch after delegation");
        let snapshot = post_delegate_next_epoch_snapshot.clone();
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

    // Show that the new validator's voting power has changed, now that we are in a new epoch
    // after the delegation.
    let new_validator_epoch_after_delegation_power = post_delegate_next_epoch_snapshot
        .get_validator_power(&new_validator_id)
        .await?
        .expect("should have voting power before delegating")
        .tap(|&power| {
            assert!(
                power > new_validator_original_power,
                "new validator should now have more voting power after receiving a delegation"
            )
        });

    // Build a transaction that will now undelegate from the validator.
    let plan = {
        use {
            penumbra_sdk_sct::component::clock::EpochRead,
            penumbra_sdk_shielded_pool::{OutputPlan, SpendPlan},
            penumbra_sdk_stake::DelegationToken,
            penumbra_sdk_transaction::{
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
            test_keys::ADDRESS_1.deref().clone(),
        );
        TransactionPlan {
            actions: vec![spend.into(), output.into(), undelegate.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(test_keys::ADDRESS_0.deref().clone()),
            )),
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
        .with_populated_detection_data(OsRng, Default::default())
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
        use penumbra_sdk_stake::{component::ConsensusIndexRead, validator::State};
        let snapshot = post_undelegate_snapshot.clone();
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

    // Compute the current voting power, confirm that it has not changed yet.
    post_undelegate_snapshot
        .get_validator_power(&new_validator_id)
        .await?
        .expect("should have voting power before delegating")
        .tap(|&power| {
            assert_eq!(
                power, new_validator_epoch_after_delegation_power,
                "validator power should not change immediately after an undelegation"
            )
        });

    // Fast forward to the next epoch.
    node.fast_forward_to_next_epoch(&storage).await?;
    let post_undelegate_next_epoch_snapshot = storage.latest_snapshot();

    // Show that after undelegating, the validator is no longer marked active.
    {
        use penumbra_sdk_stake::{component::ConsensusIndexRead, validator::State};
        info!("checking consensus set in epoch after undelegation");
        let snapshot = post_undelegate_next_epoch_snapshot.clone();
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

    // Show that now, the validator's voting power has returned to its original state.
    post_undelegate_next_epoch_snapshot
        .get_validator_power(&new_validator_id)
        .await?
        .expect("should have voting power before delegating")
        .tap(|&power| {
            assert_eq!(
                power, new_validator_original_power,
                "validator power should not change immediately after an undelegation"
            )
        });

    // The test passed. Free our temporary storage and drop our tracing subscriber.
    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
