use blake2b_simd::Params;
use decaf377_fmd::Clue;
use penumbra_chain::EffectHash;
use penumbra_dao::{DaoDeposit, DaoOutput, DaoSpend};
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw},
    swap, swap_claim,
};
use penumbra_fee::Fee;
use penumbra_ibc::Ics20Withdrawal;
use penumbra_keys::{FullViewingKey, PayloadKey};
use penumbra_proto::{
    core::crypto::v1alpha1 as pbc, core::dex::v1alpha1 as pbd, core::governance::v1alpha1 as pbg,
    core::ibc::v1alpha1 as pbi, core::stake::v1alpha1 as pbs, core::transaction::v1alpha1 as pbt,
    Message,
};
use penumbra_proto::{DomainType, TypeUrl};
use penumbra_shielded_pool::{output, spend};
use penumbra_stake::{Delegate, Undelegate, UndelegateClaimBody};

use crate::{
    action::{
        DelegatorVote, DelegatorVoteBody, Proposal, ProposalDepositClaim, ProposalSubmit,
        ProposalWithdraw, ValidatorVote, ValidatorVoteBody, Vote,
    },
    memo::MemoCiphertext,
    plan::TransactionPlan,
    transaction::DetectionData,
    Action, Transaction, TransactionBody, TransactionParameters,
};

// Note: temporarily duplicate of chain/EffectingData
pub trait EffectingData {
    fn effect_hash(&self) -> EffectHash;
}

impl<'a, T: penumbra_chain::EffectingData> EffectingData for crate::Compat<'a, T> {
    fn effect_hash(&self) -> EffectHash {
        self.0.effect_hash()
    }
}

impl Transaction {
    pub fn effect_hash(&self) -> EffectHash {
        self.transaction_body.effect_hash()
    }
}

impl TransactionBody {
    pub fn expiry_height(&self) -> u64 {
        self.transaction_parameters.expiry_height
    }

    pub fn chain_id(&self) -> &str {
        &self.transaction_parameters.chain_id
    }

    pub fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:tx_body")
            .to_state();

        // Hash the fixed data of the transaction body.
        state.update(self.transaction_parameters.effect_hash().as_bytes());
        state.update(self.fee.effect_hash().as_bytes());
        if self.memo.is_some() {
            let memo_ciphertext = self.memo.clone();
            state.update(
                memo_ciphertext
                    .expect("memo is some")
                    .effect_hash()
                    .as_bytes(),
            );
        }
        if self.detection_data.is_some() {
            let detection_data = self.detection_data.clone();
            state.update(
                detection_data
                    .expect("detection data is some")
                    .effect_hash()
                    .as_bytes(),
            );
        }

        // Hash the number of actions, then each action.
        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());
        for action in &self.actions {
            state.update(action.effect_hash().as_bytes());
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl TransactionPlan {
    /// Computes the [`EffectHash`] for the [`Transaction`] described by this
    /// [`TransactionPlan`].
    ///
    /// This method does not require constructing the entire [`Transaction`],
    /// but it does require the associated [`FullViewingKey`] to derive
    /// effecting data that will be fed into the [`EffectHash`].
    pub fn effect_hash(&self, fvk: &FullViewingKey) -> EffectHash {
        // This implementation is identical to the one above, except that we
        // don't need to actually construct the entire `TransactionBody` with
        // complete `Action`s, we just need to construct the bodies of the
        // actions the transaction will have when constructed.

        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:tx_body")
            .to_state();

        // Hash the fixed data of the transaction body.
        let tx_params = TransactionParameters {
            chain_id: self.chain_id.clone(),
            expiry_height: self.expiry_height,
        };
        state.update(tx_params.effect_hash().as_bytes());
        state.update(self.fee.effect_hash().as_bytes());

        // Hash the memo and save the memo key for use with outputs later.
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self.memo_plan.clone().unwrap();
            let memo_ciphertext = memo_plan.memo().expect("can compute ciphertext");
            state.update(memo_ciphertext.effect_hash().as_bytes());
            memo_key = Some(memo_plan.key);
        }

        // Hash the detection data.
        if !self.clue_plans.is_empty() {
            let detection_data = DetectionData {
                fmd_clues: self
                    .clue_plans
                    .iter()
                    .map(|clue_plan| clue_plan.clue())
                    .collect(),
            };
            state.update(detection_data.effect_hash().as_bytes());
        }

        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());

        // TransactionPlan::build builds the actions sorted by type, so hash the
        // actions in the order they'll appear in the final transaction.
        for spend in self.spend_plans() {
            state.update(spend.spend_body(fvk).effect_hash().as_bytes());
        }

        // If the memo_key is None, then there is no memo, and we populate the memo key
        // field with a dummy key.
        let dummy_payload_key: PayloadKey = [0u8; 32].into();
        for output in self.output_plans() {
            state.update(
                output
                    .output_body(
                        fvk.outgoing(),
                        memo_key.as_ref().unwrap_or(&dummy_payload_key),
                    )
                    .effect_hash()
                    .as_bytes(),
            );
        }
        for swap in self.swap_plans() {
            state.update(swap.swap_body(fvk).effect_hash().as_bytes());
        }
        for swap_claim in self.swap_claim_plans() {
            state.update(swap_claim.swap_claim_body(fvk).effect_hash().as_bytes());
        }
        for delegation in self.delegations() {
            state.update(delegation.effect_hash().as_bytes());
        }
        for undelegation in self.undelegations() {
            state.update(undelegation.effect_hash().as_bytes());
        }
        for plan in self.undelegate_claim_plans() {
            state.update(plan.undelegate_claim_body().effect_hash().as_bytes());
        }
        for proposal_submit in self.proposal_submits() {
            state.update(proposal_submit.effect_hash().as_bytes());
        }
        for proposal_withdraw in self.proposal_withdraws() {
            state.update(proposal_withdraw.effect_hash().as_bytes());
        }
        for validator_vote in self.validator_votes() {
            state.update(validator_vote.effect_hash().as_bytes());
        }
        for delegator_vote in self.delegator_vote_plans() {
            state.update(
                delegator_vote
                    .delegator_vote_body(fvk)
                    .effect_hash()
                    .as_bytes(),
            );
        }
        for proposal_deposit_claim in self.proposal_deposit_claims() {
            state.update(proposal_deposit_claim.effect_hash().as_bytes());
        }
        // These are data payloads, so just hash them directly,
        // since they are effecting data.
        for payload in self.validator_definitions() {
            let effect_hash = Params::default()
                .personal(b"PAH:valdefnition")
                .hash(&payload.encode_to_vec());
            state.update(effect_hash.as_bytes());
        }
        for payload in self.ibc_actions() {
            let effect_hash = Params::default()
                .personal(b"PAH:ibc_action")
                .hash(&payload.encode_to_vec());
            state.update(effect_hash.as_bytes());
        }
        for dao_spend in self.dao_spends() {
            state.update(dao_spend.effect_hash().as_bytes());
        }
        for dao_output in self.dao_outputs() {
            state.update(dao_output.effect_hash().as_bytes());
        }
        for dao_deposit in self.dao_deposits() {
            state.update(dao_deposit.effect_hash().as_bytes());
        }
        for position_open in self.position_openings() {
            state.update(position_open.effect_hash().as_bytes());
        }
        for position_close in self.position_closings() {
            state.update(position_close.effect_hash().as_bytes());
        }
        for position_withdraw in self.position_withdrawals() {
            state.update(
                position_withdraw
                    .position_withdraw()
                    .effect_hash()
                    .as_bytes(),
            );
        }
        for ics20_withdrawal in self.ics20_withdrawals() {
            state.update(ics20_withdrawal.effect_hash().as_bytes());
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Action {
    fn effect_hash(&self) -> EffectHash {
        match self {
            Action::Output(output) => output.body.effect_hash(),
            Action::Spend(spend) => spend.body.effect_hash(),
            Action::Delegate(delegate) => delegate.effect_hash(),
            Action::Undelegate(undelegate) => undelegate.effect_hash(),
            Action::UndelegateClaim(claim) => claim.body.effect_hash(),
            Action::ProposalSubmit(submit) => submit.effect_hash(),
            Action::ProposalWithdraw(withdraw) => withdraw.effect_hash(),
            Action::ProposalDepositClaim(claim) => claim.effect_hash(),
            Action::DelegatorVote(vote) => vote.effect_hash(),
            Action::ValidatorVote(vote) => vote.effect_hash(),
            Action::SwapClaim(swap_claim) => swap_claim.body.effect_hash(),
            Action::Swap(swap) => swap.body.effect_hash(),
            // These are data payloads, so just hash them directly,
            // since we consider them authorizing data.
            Action::ValidatorDefinition(payload) => EffectHash(
                Params::default()
                    .personal(b"PAH:valdefnition")
                    .hash(&payload.encode_to_vec())
                    .as_array()
                    .clone(),
            ),
            Action::IbcAction(payload) => EffectHash(
                Params::default()
                    .personal(b"PAH:ibc_action")
                    .hash(&payload.encode_to_vec())
                    .as_array()
                    .clone(),
            ),
            Action::PositionOpen(p) => p.effect_hash(),
            Action::PositionClose(p) => p.effect_hash(),
            Action::PositionWithdraw(p) => p.effect_hash(),
            Action::PositionRewardClaim(p) => p.effect_hash(),
            Action::Ics20Withdrawal(w) => w.effect_hash(),
            Action::DaoSpend(d) => d.effect_hash(),
            Action::DaoOutput(d) => d.effect_hash(),
            Action::DaoDeposit(d) => d.effect_hash(),
        }
    }
}

/// A helper function to hash the data of a proto-encoded message, using
/// the variable-length `TypeUrl` of the corresponding domain type as a
/// personalization string.
fn hash_proto_effecting_data<M: Message>(personalization: &str, message: &M) -> EffectHash {
    let mut state = blake2b_simd::State::new();

    // The `TypeUrl` provided as a personalization string is variable length,
    // so we first include the length in bytes as a fixed-length prefix.
    let length = personalization.len() as u64;
    state.update(&length.to_le_bytes());
    state.update(personalization.as_bytes());
    state.update(&message.encode_to_vec());

    EffectHash(*state.finalize().as_array())
}

impl EffectingData for Ics20Withdrawal {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbi::Ics20Withdrawal = self.clone().into();
        hash_proto_effecting_data(Ics20Withdrawal::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for output::Body {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the output, so we can
        // just use hash the proto-encoding of the body.
        let body: pbt::OutputBody = self.clone().into();
        hash_proto_effecting_data(output::Body::TYPE_URL, &body)
    }
}

impl EffectingData for spend::Body {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the spend, so we can
        // just use hash the proto-encoding of the body.
        let body: pbt::SpendBody = self.clone().into();
        hash_proto_effecting_data(spend::Body::TYPE_URL, &body)
    }
}

impl EffectingData for DaoDeposit {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::DaoDeposit = self.clone().into();
        hash_proto_effecting_data(DaoDeposit::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for DaoSpend {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::DaoSpend = self.clone().into();
        hash_proto_effecting_data(DaoSpend::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for DaoOutput {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::DaoOutput = self.clone().into();
        hash_proto_effecting_data(DaoOutput::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for swap::Body {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the swap, so we can
        // just use hash the proto-encoding of the body.
        let effecting_data: pbd::SwapBody = self.clone().into();
        hash_proto_effecting_data(swap::Body::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for swap_claim::Body {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the swap claim, so we can
        // just use hash the proto-encoding of the body.
        let effecting_data: pbd::SwapClaimBody = self.clone().into();
        hash_proto_effecting_data(swap_claim::Body::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for Delegate {
    fn effect_hash(&self) -> EffectHash {
        // For delegations, the entire action is considered effecting data.
        let effecting_data: pbs::Delegate = self.clone().into();
        hash_proto_effecting_data(Delegate::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for Undelegate {
    fn effect_hash(&self) -> EffectHash {
        // For undelegations, the entire action is considered effecting data.
        let effecting_data: pbs::Undelegate = self.clone().into();
        hash_proto_effecting_data(Undelegate::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for UndelegateClaimBody {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the undelegate claim, so we can
        // just use hash the proto-encoding of the body.
        let effecting_data: pbs::UndelegateClaimBody = self.clone().into();
        hash_proto_effecting_data(UndelegateClaimBody::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for Proposal {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::Proposal = self.clone().into();
        hash_proto_effecting_data(Proposal::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for ProposalSubmit {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::ProposalSubmit = self.clone().into();
        hash_proto_effecting_data(ProposalSubmit::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for ProposalWithdraw {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::ProposalWithdraw = self.clone().into();
        hash_proto_effecting_data(ProposalWithdraw::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for ValidatorVote {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
    }
}

impl EffectingData for DelegatorVote {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
    }
}

impl EffectingData for Vote {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::Vote = self.clone().into();
        hash_proto_effecting_data(Vote::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for ValidatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::ValidatorVoteBody = self.clone().into();
        hash_proto_effecting_data(ValidatorVoteBody::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for DelegatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::DelegatorVoteBody = self.clone().into();
        hash_proto_effecting_data(DelegatorVoteBody::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for ProposalDepositClaim {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbg::ProposalDepositClaim = self.clone().into();
        hash_proto_effecting_data(ProposalDepositClaim::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for PositionOpen {
    fn effect_hash(&self) -> EffectHash {
        // The position open action consists only of the position, which
        // we consider effecting data.
        let effecting_data: pbd::PositionOpen = self.clone().into();
        hash_proto_effecting_data(PositionOpen::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for PositionClose {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbd::PositionClose = self.clone().into();
        hash_proto_effecting_data(PositionClose::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for PositionWithdraw {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbd::PositionWithdraw = self.clone().into();
        hash_proto_effecting_data(PositionWithdraw::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for PositionRewardClaim {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pbd::PositionRewardClaim = self.clone().into();
        hash_proto_effecting_data(PositionRewardClaim::TYPE_URL, &effecting_data)
    }
}

impl EffectingData for DetectionData {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:detect_data")
            .to_state();

        let num_clues = self.fmd_clues.len() as u32;
        state.update(&num_clues.to_le_bytes());
        for fmd_clue in &self.fmd_clues {
            state.update(fmd_clue.effect_hash().as_bytes());
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Clue {
    fn effect_hash(&self) -> EffectHash {
        let data: pbc::Clue = self.clone().into();
        hash_proto_effecting_data(Clue::TYPE_URL, &data)
    }
}

impl EffectingData for TransactionParameters {
    fn effect_hash(&self) -> EffectHash {
        let params: pbt::TransactionParameters = self.clone().into();
        hash_proto_effecting_data(TransactionParameters::TYPE_URL, &params)
    }
}

impl EffectingData for Fee {
    fn effect_hash(&self) -> EffectHash {
        let proto_encoded_fee: pbc::Fee = self.clone().into();
        hash_proto_effecting_data(Fee::TYPE_URL, &proto_encoded_fee)
    }
}

impl EffectingData for MemoCiphertext {
    fn effect_hash(&self) -> EffectHash {
        let proto_encoded_memo: pbt::MemoCiphertext = self.clone().into();
        hash_proto_effecting_data(MemoCiphertext::TYPE_URL, &proto_encoded_memo)
    }
}

#[cfg(test)]
mod tests {
    use penumbra_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
    use penumbra_dex::{swap::SwapPlaintext, swap::SwapPlan, TradingPair};
    use penumbra_fee::Fee;
    use penumbra_keys::{
        keys::{SeedPhrase, SpendKey},
        Address,
    };
    use penumbra_shielded_pool::Note;
    use penumbra_shielded_pool::{OutputPlan, SpendPlan};
    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::{
        memo::MemoPlaintext,
        plan::{CluePlan, MemoPlan, TransactionPlan},
        WitnessData,
    };

    /// This isn't an exhaustive test, but we don't currently have a
    /// great way to generate actions for randomized testing.
    ///
    /// All we hope to check here is that, for a basic transaction plan,
    /// we compute the same auth hash for the plan and for the transaction.
    #[test]
    fn plan_effect_hash_matches_transaction_effect_hash() {
        let rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let (addr, _dtk) = fvk.incoming().payment_address(0u32.into());

        let mut sct = tct::Tree::new();

        let note0 = Note::generate(
            &mut OsRng,
            &addr,
            Value {
                amount: 10000u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );
        let note1 = Note::generate(
            &mut OsRng,
            &addr,
            Value {
                amount: 20000u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );

        sct.insert(tct::Witness::Keep, note0.commit()).unwrap();
        sct.insert(tct::Witness::Keep, note1.commit()).unwrap();

        let trading_pair = TradingPair::new(
            asset::Cache::with_known_assets()
                .get_unit("nala")
                .unwrap()
                .id(),
            asset::Cache::with_known_assets()
                .get_unit("upenumbra")
                .unwrap()
                .id(),
        );

        let swap_plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            100000u64.into(),
            1u64.into(),
            Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .unwrap()
                    .id(),
            }),
            addr,
        );

        let mut rng = OsRng;

        let plan = TransactionPlan {
            expiry_height: 0,
            fee: Fee::default(),
            chain_id: "penumbra-test".to_string(),
            // Put outputs first to check that the auth hash
            // computation is not affected by plan ordering.
            actions: vec![
                OutputPlan::new(
                    &mut OsRng,
                    Value {
                        amount: 30000u64.into(),
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    addr.clone(),
                )
                .into(),
                SpendPlan::new(&mut OsRng, note0, 0u64.into()).into(),
                SpendPlan::new(&mut OsRng, note1, 1u64.into()).into(),
                SwapPlan::new(&mut OsRng, swap_plaintext).into(),
            ],
            clue_plans: vec![CluePlan::new(&mut OsRng, addr, 1)],
            memo_plan: Some(
                MemoPlan::new(
                    &mut OsRng,
                    MemoPlaintext {
                        sender: Address::dummy(&mut rng),
                        text: "".to_string(),
                    },
                )
                .unwrap(),
            ),
        };

        println!("{}", serde_json::to_string_pretty(&plan).unwrap());

        let plan_effect_hash = plan.effect_hash(fvk);

        let auth_data = plan.authorize(rng, &sk);
        let witness_data = WitnessData {
            anchor: sct.root(),
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend: &SpendPlan| {
                    (
                        spend.note.commit(),
                        sct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let transaction = plan
            .build(fvk, witness_data)
            .unwrap()
            .authorize(&mut OsRng, &auth_data)
            .unwrap();

        let transaction_effect_hash = transaction.effect_hash();

        assert_eq!(plan_effect_hash, transaction_effect_hash);

        // TODO: fix this and move into its own test?
        // // Also check the concurrent build results in the same effect hash.
        // let rt = Runtime::new().unwrap();
        // let transaction = rt
        //     .block_on(async move {
        //         plan.build_concurrent(&mut OsRng, fvk, auth_data, witness_data)
        //             .await
        //     })
        //     .expect("can build");
        // assert_eq!(plan_effect_hash, transaction.effect_hash());
    }
}
