use {
    anyhow::anyhow,
    cnidarium::TempStorage,
    decaf377_rdsa::VerificationKey,
    penumbra_app::{
        genesis::{AppState, Content},
        server::consensus::Consensus,
    },
    penumbra_community_pool::{
        CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend, StateReadExt,
    },
    penumbra_governance::{
        Proposal, ProposalSubmit, ValidatorVote, ValidatorVoteBody, ValidatorVoteReason,
    },
    penumbra_keys::{
        keys::{SpendKey, SpendKeyBytes},
        test_keys::{self},
    },
    penumbra_mock_client::MockClient,
    penumbra_mock_consensus::TestNode,
    penumbra_proto::{
        core::keys::v1::{GovernanceKey, IdentityKey},
        penumbra::core::component::stake::v1::Validator as PenumbraValidator,
        DomainType,
    },
    penumbra_shielded_pool::{genesis::Allocation, OutputPlan, SpendPlan},
    penumbra_stake::{component::validator_handler::ValidatorDataRead, DelegationToken},
    penumbra_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, ActionPlan, TransactionParameters, TransactionPlan,
    },
    rand::Rng,
    rand_core::OsRng,
    std::collections::BTreeMap,
    tap::{Tap, TapFallible},
    tracing::{error_span, info, Instrument},
};

mod common;

const PROPOSAL_VOTING_BLOCKS: u64 = 3;

/// Exercises that the app can make proposals to spend community pool funds.
#[tokio::test]
async fn app_can_propose_community_pool_spends() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new().await?;

    // Define a helper to get the current community pool balance.
    let pool_balance = || async { storage.latest_snapshot().community_pool_balance().await };

    // Generate a set of consensus keys.
    let consensus_sk = ed25519_consensus::SigningKey::new(OsRng);
    let consensus_vk = consensus_sk.verification_key();

    // Generate a set of identity keys.
    let spend_key: SpendKey = SpendKeyBytes(OsRng.gen()).into();
    let (identity_sk, identity_vk) = {
        let sk = spend_key.spend_auth_key();
        let vk = VerificationKey::from(sk);
        (sk, vk)
    };
    let (governance_sk, governance_vk) = (identity_sk, identity_vk);

    // Define a validator and an associated genesis allocation.
    let (validator, allocation) = {
        let v = PenumbraValidator {
            identity_key: Some(IdentityKey {
                ik: identity_vk.to_bytes().to_vec(),
            }),
            // NB: for now, we will use the same key for governance. See the documentation of
            // `GovernanceKey` for more information about cold storage of validator keys.
            governance_key: Some(GovernanceKey {
                gk: identity_vk.to_bytes().to_vec(),
            }),
            consensus_key: consensus_vk.as_bytes().to_vec(),
            enabled: true,
            sequence_number: 0,
            name: String::default(),
            website: String::default(),
            description: String::default(),
            funding_streams: Vec::default(),
        };

        let (address, _) = spend_key
            .full_viewing_key()
            .incoming()
            .payment_address(0u32.into());

        let ik = penumbra_stake::IdentityKey(identity_vk.into());
        let delegation_denom = DelegationToken::from(ik).denom();

        let allocation = Allocation {
            raw_amount: 1000u128.into(),
            raw_denom: delegation_denom.to_string(),
            address,
        };

        (v, allocation)
    };

    // Define our application state, and start the test node.
    let mut test_node = {
        let mut content = Content {
            governance_content: penumbra_governance::genesis::Content {
                governance_params: penumbra_governance::params::GovernanceParameters {
                    proposal_deposit_amount: 0_u32.into(),
                    proposal_voting_blocks: PROPOSAL_VOTING_BLOCKS,
                    ..Default::default()
                },
            },
            ..Default::default()
        };
        content.stake_content.validators.push(validator);
        content.shielded_pool_content.allocations.push(allocation);
        let app_state = AppState::Content(content);
        let app_state = serde_json::to_vec(&app_state).unwrap();
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .app_state(app_state)
            .init_chain(consensus)
            .await
            .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))?
    };
    let [_validator] = storage
        .latest_snapshot()
        .validator_definitions()
        .await?
        .try_into()
        .map_err(|validator| anyhow::anyhow!("expected one validator, got: {validator:?}"))?;

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
    plan.populate_detection_data(OsRng, 0);
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .instrument(error_span!("executing block with community pool deposit"))
        .await?;
    let post_deposit_pool_balance = pool_balance().await?;

    // Now, make a governance proposal that we should spend community pool funds, to return
    // the note back to the test wallet.
    let mut plan = {
        let value = note.value();
        let proposed_tx_plan = TransactionPlan {
            actions: vec![
                CommunityPoolSpend { value }.into(),
                CommunityPoolOutput {
                    value,
                    address: *test_keys::ADDRESS_0,
                }
                .into(),
            ],
            memo: None,
            detection_data: None,
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        };
        let proposal_submit = ProposalSubmit {
            proposal: Proposal {
                id: 0_u64,
                title: "return test deposit".to_owned(),
                description: "a proposal to return the community pool deposit".to_owned(),
                payload: penumbra_governance::ProposalPayload::CommunityPoolSpend {
                    transaction_plan: proposed_tx_plan.encode_to_vec(),
                    // transaction_plan: TransactionPlan::default().encode_to_vec(),
                },
            },
            deposit_amount: 0_u32.into(),
        };
        let proposal_nft_value = proposal_submit.proposal_nft_value();
        let proposal = ActionPlan::ProposalSubmit(proposal_submit);
        TransactionPlan {
            actions: vec![
                proposal,
                // Next, create a new output of the exact same amount.
                OutputPlan::new(&mut OsRng, proposal_nft_value, *test_keys::ADDRESS_0).into(),
            ],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(*test_keys::ADDRESS_0),
            )?),
            detection_data: None,
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
    };
    plan.populate_detection_data(OsRng, 0);
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .instrument(error_span!("executing block with governance proposal"))
        .await?;
    let post_proposal_pool_balance = pool_balance().await?;

    // Now make another transaction that will contain a validator vote upon our transaction.
    let mut plan = {
        let body = ValidatorVoteBody {
            proposal: 0_u64,
            vote: penumbra_governance::Vote::Yes,
            identity_key: penumbra_stake::IdentityKey(identity_vk.to_bytes().into()),
            governance_key: penumbra_stake::GovernanceKey(governance_vk),
            reason: ValidatorVoteReason("test reason".to_owned()),
        };
        let auth_sig = governance_sk.sign(OsRng, body.encode_to_vec().as_slice());
        let vote = ValidatorVote { body, auth_sig }.into();
        TransactionPlan {
            actions: vec![vote],
            memo: None,
            detection_data: None,
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
    };
    plan.populate_detection_data(OsRng, 0);
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .instrument(error_span!("executing block with validator vote"))
        .await?;
    let post_vote_pool_balance = pool_balance().await?;

    test_node.fast_forward(PROPOSAL_VOTING_BLOCKS).await?;
    let post_voting_period_pool_balance = pool_balance().await?;

    // After we deposit a note into the community pool, we should see the original pool contents,
    // plus the amount that we deposited.
    assert_eq!(
        [(note.asset_id(), note.amount())]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        post_deposit_pool_balance,
        "a community pool deposit should be reflected in the visible balance"
    );

    // A proposal should not itself affect the balance of the community pool.
    assert_eq!(
        post_deposit_pool_balance, post_proposal_pool_balance,
        "the community pool balance should not be affected by a proposal"
    );

    // ...nor should a vote by itself.
    assert_eq!(
        post_proposal_pool_balance, post_vote_pool_balance,
        "the community pool balance should not be affected by a vote, even with quorum"
    );

    // After the proposal passes, we should see the balance decrease by the amount proposed.
    assert_eq!(
        BTreeMap::default(),
        post_voting_period_pool_balance,
        "the successful proposal should decrease the funds of the community pool"
    );

    // Free our temporary storage.
    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
