use {
    self::common::ValidatorDataReadExt,
    anyhow::anyhow,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    decaf377_rdsa::VerificationKey,
    penumbra_sdk_app::{
        genesis::{AppState, Content},
        server::consensus::Consensus,
        CommunityPoolStateReadExt as _,
    },
    penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID,
    penumbra_sdk_community_pool::{
        CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend, StateReadExt as _,
    },
    penumbra_sdk_governance::{
        Proposal, ProposalSubmit, StateReadExt as _, ValidatorVote, ValidatorVoteBody,
        ValidatorVoteReason,
    },
    penumbra_sdk_keys::{
        keys::{SpendKey, SpendKeyBytes},
        test_keys,
    },
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_num::Amount,
    penumbra_sdk_proto::{
        core::keys::v1::{GovernanceKey, IdentityKey},
        penumbra::core::component::stake::v1::Validator as PenumbraValidator,
        DomainType,
    },
    penumbra_sdk_shielded_pool::{genesis::Allocation, OutputPlan, SpendPlan},
    penumbra_sdk_stake::DelegationToken,
    penumbra_sdk_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, ActionPlan, TransactionParameters, TransactionPlan,
    },
    rand::Rng,
    rand_core::OsRng,
    std::{collections::BTreeMap, ops::Deref},
    tap::{Tap, TapFallible},
    tracing::{error_span, info, Instrument},
};

mod common;

const PROPOSAL_VOTING_BLOCKS: u64 = 3;

/// Exercises that the app can disable proposals to spend community pool funds.
#[tokio::test]
async fn app_can_disable_community_pool_spends() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();
    let storage = TempStorage::new_with_penumbra_prefixes().await?;

    // Define a helper to get the current community pool balance.
    let pool_balance = || async { storage.latest_snapshot().community_pool_balance().await };
    let pending_pool_txs = || async {
        storage
            .latest_snapshot()
            .pending_community_pool_transactions()
            .await
    };

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

        let ik = penumbra_sdk_stake::IdentityKey(identity_vk.into());
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
            chain_id: TestNode::<()>::CHAIN_ID.to_string(),
            governance_content: penumbra_sdk_governance::genesis::Content {
                governance_params: penumbra_sdk_governance::params::GovernanceParameters {
                    proposal_deposit_amount: 0_u32.into(),
                    proposal_voting_blocks: PROPOSAL_VOTING_BLOCKS,
                    ..Default::default()
                },
            },
            community_pool_content: penumbra_sdk_community_pool::genesis::Content {
                community_pool_params:
                    penumbra_sdk_community_pool::params::CommunityPoolParameters {
                        // Disable community spend proposals.
                        community_pool_spend_proposals_enabled: false,
                    },
                ..Default::default()
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
    let original_pool_balance = pool_balance().await?;
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
    let plan = {
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
        .with_populated_detection_data(OsRng, Default::default())
    };
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
    let plan = {
        let value = note.value();
        let proposed_tx_plan = TransactionPlan {
            actions: vec![
                CommunityPoolSpend { value }.into(),
                CommunityPoolOutput {
                    value,
                    address: test_keys::ADDRESS_0.deref().clone(),
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
                payload: penumbra_sdk_governance::ProposalPayload::CommunityPoolSpend {
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
                OutputPlan::new(
                    &mut OsRng,
                    proposal_nft_value,
                    test_keys::ADDRESS_0.deref().clone(),
                )
                .into(),
            ],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(test_keys::ADDRESS_0.deref().clone()),
            )),
            detection_data: None,
            transaction_parameters: TransactionParameters {
                chain_id: TestNode::<()>::CHAIN_ID.to_string(),
                ..Default::default()
            },
        }
        .with_populated_detection_data(OsRng, Default::default())
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .instrument(error_span!("executing block with governance proposal"))
        .await?;
    let post_proposal_pool_balance = pool_balance().await?;
    let post_proposal_pending_txs = pending_pool_txs().await?;
    let post_proposal_state = storage.latest_snapshot().proposal_state(0).await?;

    // Now make another transaction that will contain a validator vote upon our transaction.
    let plan = {
        let body = ValidatorVoteBody {
            proposal: 0_u64,
            vote: penumbra_sdk_governance::Vote::Yes,
            identity_key: penumbra_sdk_stake::IdentityKey(identity_vk.to_bytes().into()),
            governance_key: penumbra_sdk_stake::GovernanceKey(governance_vk),
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
        .with_populated_detection_data(OsRng, Default::default())
    };
    let tx = client.witness_auth_build(&plan).await?;

    // Execute the transaction, applying it to the chain state.
    test_node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .instrument(error_span!("executing block with validator vote"))
        .await?;
    let post_vote_pool_balance = pool_balance().await?;
    let post_vote_pending_txs = pending_pool_txs().await?;
    let post_vote_state = storage.latest_snapshot().proposal_state(0).await?;

    test_node.fast_forward(PROPOSAL_VOTING_BLOCKS).await?;
    let post_voting_period_pool_balance = pool_balance().await?;
    let post_voting_period_pending_txs = pending_pool_txs().await?;
    let post_voting_period_state = storage.latest_snapshot().proposal_state(0).await?;

    // At the outset, the pool should be empty.
    assert_eq!(
        original_pool_balance.len(),
        1,
        "fresh community pool only track the staking token"
    );
    assert_eq!(
        *original_pool_balance
            .get(&STAKING_TOKEN_ASSET_ID)
            .expect("CP tracks staking token, even with no balance"),
        Amount::zero(),
        "the community pool should be empty at the beginning of the chain"
    );

    // After we deposit a note into the community pool, we should see the original pool contents,
    // plus the amount that we deposited.
    assert_eq!(
        [(note.asset_id(), note.amount()),
        (*STAKING_TOKEN_ASSET_ID, Amount::zero())]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        post_deposit_pool_balance,
        "a community pool deposit should be reflected in the visible balance, even if spends are disabled"
    );

    // A proposal should not itself affect the balance of the community pool.
    assert_eq!(
        post_deposit_pool_balance, post_proposal_pool_balance,
        "the community pool balance should not be affected by a proposal"
    );
    assert_eq!(post_proposal_state, None, "the proposal should be rejected");
    assert_eq!(
        post_proposal_pending_txs.len(),
        0,
        "no transaction(s) should be pending"
    );

    // ...nor should a vote by itself.
    assert_eq!(
        post_proposal_pool_balance, post_vote_pool_balance,
        "the community pool balance should not be affected by a vote, even with quorum"
    );
    assert_eq!(
        post_vote_state, None,
        "a vote for a rejected proposal should not cause it to enter the voting state"
    );
    assert_eq!(
        post_vote_pending_txs.len(),
        0,
        "no transaction(s) should be pending"
    );

    // After any possible voting period, we should see the same pool balance.
    assert_eq!(
        post_voting_period_pool_balance,
        [
            (note.asset_id(), note.amount()),
            (*STAKING_TOKEN_ASSET_ID, Amount::zero())
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>(),
        "a rejected proposal should not decrease the funds of the community pool"
    );
    assert_eq!(
        post_voting_period_state, None,
        "a proposal should be finished after the voting period completes"
    );
    assert_eq!(
        post_voting_period_pending_txs.len(),
        0,
        "a proposal has been rejected, no transaction(s) are pending"
    );

    // Free our temporary storage.
    Ok(())
        .tap(|_| drop(test_node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
