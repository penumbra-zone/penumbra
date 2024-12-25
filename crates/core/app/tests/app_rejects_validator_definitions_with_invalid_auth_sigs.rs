use {
    self::common::{BuilderExt, ValidatorDataReadExt},
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
    penumbra_sdk_stake::{validator::Validator, FundingStreams, GovernanceKey, IdentityKey},
    rand_core::OsRng,
    tap::Tap,
    tracing::{error_span, info, Instrument},
};

mod common;

/// Show that the application rejects validator definitions with an invalid auth signature.
#[tokio::test]
async fn app_rejects_validator_definitions_with_invalid_auth_sigs() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Start the test node.
    let mut node = {
        let consensus = Consensus::new(storage.as_ref().clone());
        let app_state = AppState::Content(
            genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
        );
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
    }?;

    // Assert that there is only a single validator definition present, before we go any further.
    assert_eq!(
        storage
            .latest_snapshot()
            .validator_definitions()
            .await?
            .len(),
        1,
        "invalid validator definition transactions should not be accepted"
    );

    // Sync the mock client, using the test wallet's spend key, to the latest snapshot.
    let client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    // To define a validator, we need to define two keypairs: an identity key
    // for the Penumbra application and a consensus key for cometbft.
    let new_validator_id_sk = SigningKey::<SpendAuth>::new(OsRng);
    let new_validator_id = IdentityKey(VerificationKey::from(&new_validator_id_sk).into());
    let new_validator_consensus_sk = ed25519_consensus::SigningKey::new(OsRng);
    let new_validator_consensus = new_validator_consensus_sk.verification_key();

    // Create a different signing key, which we will use to create a forged authentication
    // signature in our validator definition transaction.
    let different_signing_key = SigningKey::<SpendAuth>::new(OsRng);

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

    // Make a transaction that defines a new validator, providing an invalid signature.
    let plan = {
        use {
            penumbra_sdk_stake::validator,
            penumbra_sdk_transaction::{ActionPlan, TransactionParameters, TransactionPlan},
            rand_core::OsRng,
        };
        let bytes = new_validator.encode_to_vec();
        // NB: we do NOT use the validator's signing key here. this transaction will contain an
        // invalid authentication signature.
        let auth_sig = different_signing_key.sign(OsRng, &bytes);
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
        plan.populate_detection_data(rand_core::OsRng, Default::default());
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

    // Assert that there is still only a single validator definition present, and that this
    // invalid transaction did not cause a new definition to appear.
    assert_eq!(
        post_tx_snapshot.validator_definitions().await?.len(),
        1,
        "invalid validator definition transactions should not be accepted"
    );

    // The test passed. Free our temporary storage and drop our tracing subscriber.
    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
