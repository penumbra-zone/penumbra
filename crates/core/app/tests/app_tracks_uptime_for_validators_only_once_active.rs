use {
    self::common::{BuilderExt, TestNodeExt, ValidatorDataReadExt},
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
    penumbra_sdk_sct::component::clock::EpochRead,
    penumbra_sdk_stake::{
        component::validator_handler::validator_store::ValidatorDataRead, validator::Validator,
        FundingStreams, GovernanceKey, IdentityKey, Uptime,
    },
    rand_core::OsRng,
    std::ops::Deref,
    tap::Tap,
    tracing::{error_span, Instrument},
};

mod common;

#[tokio::test]
async fn app_tracks_uptime_for_validators_only_once_active() -> anyhow::Result<()> {
    /// The length of the [`penumbra_sdk_sct`] epoch.
    ///
    /// This test relies on many epochs turning over, so we will work with a shorter epoch duration.
    const EPOCH_DURATION: u64 = 8;

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

    // Create a mock client.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());

    /// Helper function, retrieve a validator's [`Uptime`].
    async fn get_uptime(storage: &TempStorage, id: IdentityKey) -> Option<Uptime> {
        storage
            .latest_snapshot()
            .get_validator_uptime(&id)
            .await
            .expect("should be able to get a validator uptime")
    }

    // Helper function, count the validators in the current consensus set.
    let get_latest_consensus_set = || async {
        use penumbra_sdk_stake::component::ConsensusIndexRead;
        storage
            .latest_snapshot()
            .get_consensus_set()
            .await
            .expect("latest snapshot should have a valid consensus set")
    };

    // Get the identity key of the genesis validator, before we go further.
    let [existing_validator_id] = storage
        .latest_snapshot()
        .validator_identity_keys()
        .await?
        .try_into()
        .map_err(|keys| anyhow::anyhow!("expected one key, got: {keys:?}"))?;

    // To define a validator, we need to define two keypairs: an identity key
    // for the Penumbra application and a consensus key for cometbft.
    let new_validator_id_sk = SigningKey::<SpendAuth>::new(OsRng);
    let new_validator_id = IdentityKey(VerificationKey::from(&new_validator_id_sk).into());
    let new_validator_consensus_sk = ed25519_consensus::SigningKey::new(OsRng);
    let new_validator_consensus = new_validator_consensus_sk.verification_key();

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
    let new_validator_id = new_validator.identity_key;

    // Helper functions, retrieve validators' [`Uptime`].
    let existing_validator_uptime = || get_uptime(&storage, existing_validator_id);
    let new_validator_uptime = || get_uptime(&storage, new_validator_id);

    // Make a transaction that defines the new validator.
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

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(client.witness_auth_build(&plan).await?.encode_to_vec())
        .execute()
        .instrument(error_span!(
            "executing block with validator definition transaction"
        ))
        .await?;

    // Show that we do not yet have uptime data for the new validator. It is defined, but its
    // uptime is not being tracked yet because it is not part of the consensus set.
    {
        assert_eq!(
            get_latest_consensus_set().await.len(),
            1,
            "there should only be one validator in the consensus set"
        );
        assert!(new_validator_uptime().await.is_none());
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
            .ok_or(anyhow::anyhow!("new validator has a rate"))?
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

    // Execute the delegation transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with delegation transaction"))
        .await?;

    // The new validator uptime should not yet be in effect.
    {
        assert_eq!(
            get_latest_consensus_set().await.len(),
            1,
            "there should only be one validator in the consensus set"
        );
        assert!(new_validator_uptime().await.is_none());
    }

    // Fast forward to the next epoch.
    node.fast_forward_to_next_epoch(&storage).await?;

    // The new validator should now be in the consensus set.
    {
        assert_eq!(
            get_latest_consensus_set().await.len(),
            2,
            "the delegated validator should now be participating in consensus"
        );
    }

    // Show that the uptime trackers look correct for the genesis validator and for our newly
    // active validator.
    {
        let new = new_validator_uptime().await.expect("uptime should exist");
        let existing = existing_validator_uptime()
            .await
            .expect("uptime should exist");
        assert_eq!(
            new.num_missed_blocks(),
            // FIXME: promoted validators always miss one block.
            // > "tracking is done at the beginning of block execution, and based on the previous
            // > block commit h-1 so if a validator is promoted into the active set at h it will
            // > always have 1 missed bock - not sure this is worth fixing"
            // - @erwanor
            1,
            "newly active validator has missed no blocks"
        );
        assert_eq!(
            new.as_of_height(),
            storage.latest_snapshot().get_block_height().await?,
            "validators' uptime trackers are up-to-date with latest height"
        );
        assert_eq!(
            new.as_of_height(),
            existing.as_of_height(),
            "both validators' uptime trackers are equally recent"
        );
        assert_eq!(
            existing.num_missed_blocks(),
            0,
            "genesis validator has signed all blocks in the previous epoch"
        );
    }

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
            .ok_or(anyhow::anyhow!("new validator has a rate"))?
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

    // Execute the undelegation transaction, applying it to the chain state.
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("executing block with undelegation transaction"))
        .await?;

    // We should not yet see the uptime tracker disappear.
    assert!(
        new_validator_uptime()
            .await
            .is_some_and(|u| u.num_missed_blocks() == 2),
        "new validator uptime should still be tracked after undelegation"
    );

    // Fast forward to the next epoch.
    node.fast_forward_to_next_epoch(&storage).await?;

    // The validator should no longer be part of the consensus set.
    {
        assert_eq!(
            get_latest_consensus_set().await.len(),
            1,
            "the undelegated validator should no longer be participating in consensus"
        );
    }

    // Now that the validator is no longer a part of the consensus set, there is not a
    // corresponding uptime tracker.
    {
        let new = new_validator_uptime().await.expect("uptime should exist");
        let existing = existing_validator_uptime()
            .await
            .expect("uptime should exist");
        assert_eq!(
            new.as_of_height(),
            EPOCH_DURATION * 2 - 1,
            "new validator uptime is not updated after leaving consensus"
        );
        assert_eq!(
            existing.as_of_height(),
            EPOCH_DURATION * 2,
            "active validators' uptime is still tracked"
        );
    }

    // Fast forward to the next epoch.
    node.fast_forward_to_next_epoch(&storage).await?;

    // There should only be one validator in the consensus set.
    {
        assert_eq!(
            get_latest_consensus_set().await.len(),
            1,
            "the undelegated validator should no longer be participating in consensus"
        );
    }

    // Even as we continue into the next epoch, we will continue only to update the active
    // validators' uptimes.
    {
        let new = new_validator_uptime().await.expect("uptime should exist");
        let existing = existing_validator_uptime()
            .await
            .expect("uptime should exist");
        assert_eq!(
            new.as_of_height(),
            EPOCH_DURATION * 2 - 1,
            "new validator uptime is still not updated after leaving consensus"
        );
        assert_eq!(
            existing.as_of_height(),
            EPOCH_DURATION * 3,
            "active validators' uptime will continue to be tracked"
        );
    }

    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
