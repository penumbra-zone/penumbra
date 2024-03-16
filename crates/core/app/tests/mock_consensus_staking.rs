use {
    self::common::BuilderExt,
    anyhow::Context,
    cnidarium::TempStorage,
    penumbra_app::server::consensus::Consensus,
    penumbra_genesis::AppState,
    penumbra_keys::test_keys,
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_stake::component::validator_handler::ValidatorDataRead as _,
    tap::Tap,
    tracing::{error_span, info, Instrument},
};

mod common;

#[tokio::test]
async fn mock_consensus_can_delegate_to_a_validator() -> anyhow::Result<()> {
    // Install a test logger, acquire some temporary storage, and start the test node.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Start the test node.
    let mut node = {
        let consensus = Consensus::new(storage.as_ref().clone());
        let app_state = AppState::default();
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
    }?;

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // TODO(kate): get this number by querying the chain parameters.
    const EPOCH_LENGTH: usize = 1000;

    // Fast forward to the next epoch.
    let snapshot_start = storage.latest_snapshot();
    node.fast_forward(EPOCH_LENGTH)
        .instrument(error_span!("fast forwarding test node to next epoch"))
        .await
        .context("fast forwarding {EPOCH_LENGTH} blocks")?;
    let snapshot_end = storage.latest_snapshot();

    // Retrieve the validator definition from the latest snapshot.
    let validator = match snapshot_end
        .validator_definitions()
        .tap(|_| info!("getting validator definitions"))
        .await?
        .as_slice()
    {
        [v] => v.clone(),
        unexpected => panic!("there should be one validator, got: {unexpected:?}"),
    };

    let penumbra_stake::validator::Validator { identity_key, .. } = validator;

    // Check that we are now in a new epoch.
    {
        use penumbra_sct::{component::clock::EpochRead, epoch::Epoch};
        let Epoch { index: start, .. } = snapshot_start.get_current_epoch().await?;
        let Epoch { index: end, .. } = snapshot_end.get_current_epoch().await?;
        assert_eq!(start, 0, "we should start in the first epoch");
        assert_eq!(end, 1, "we should now be in the second epoch");
    }

    // Show that the validator is and was active.
    {
        use penumbra_stake::{component::validator_handler::ValidatorDataRead, validator::State};
        let start = snapshot_start.get_validator_state(&identity_key).await?;
        let end = snapshot_end.get_validator_state(&identity_key).await?;
        assert_eq!(start, Some(State::Active));
        assert_eq!(end, Some(State::Active));
    }

    async fn get_consensus_set(
        snapshot: &cnidarium::Snapshot,
    ) -> anyhow::Result<Vec<penumbra_stake::IdentityKey>> {
        use futures::TryStreamExt;
        use penumbra_stake::component::ConsensusIndexRead;
        snapshot.consensus_set_stream()?.try_collect().await
    }

    // Show that the validator was, and still is, in the consensus set.
    {
        let start = get_consensus_set(&snapshot_start).await?;
        let end = get_consensus_set(&snapshot_end).await?;
        let expected = [identity_key];
        assert_eq!(
            start, expected,
            "validator should start in the consensus set"
        );
        assert_eq!(end, expected, "validator should stay in the consensus set");
    }

    // Make a transaction that defines a new validator.
    let new_validator_vk = node.keyring_mut().add_key();
    let (new_validator_pb, _, new_validator_spend_key) =
        self::common::generate_penumbra_validator(&new_validator_vk);
    let new_validator = penumbra_stake::validator::Validator::try_from(new_validator_pb.clone())?;
    let mut plan = {
        use {
            penumbra_proto::Message,
            penumbra_stake::validator,
            penumbra_transaction::{ActionPlan, TransactionParameters, TransactionPlan},
            rand_core::OsRng,
        };
        let bytes = new_validator_pb.encode_to_vec();
        let auth_sig = new_validator_spend_key.spend_auth_key().sign(OsRng, &bytes);
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
    };
    plan.populate_detection_data(rand_core::OsRng, 0);
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    node.block()
        .add_tx(penumbra_proto::DomainType::encode_to_vec(&tx))
        .execute()
        .await?;
    let post_tx_snapshot = storage.latest_snapshot();

    // Show that the set of validators looks correct.
    {
        use penumbra_stake::validator::State;
        let snapshot = post_tx_snapshot;
        // The original validator should still be active.
        assert_eq!(
            snapshot.get_validator_state(&identity_key).await?,
            Some(State::Active),
            "validator should be active"
        );
        // The new validator should be defined, but not yet active. It should not be inclueded in
        // consensus yet.
        assert_eq!(
            snapshot
                .get_validator_state(&new_validator.identity_key)
                .await?,
            Some(State::Defined),
            "new validator definition should be defined but not active"
        );
        // The original validator should still be the only validator in the consensus set.
        assert_eq!(
            get_consensus_set(&snapshot_start).await?.len(),
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
