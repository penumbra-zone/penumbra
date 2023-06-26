use blake2b_simd::{Hash, Params};
use decaf377::FieldExt;
use decaf377_fmd::Clue;
use penumbra_crypto::{EffectHash, Fee, FullViewingKey, NotePayload, PayloadKey};
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen, PositionRewardClaim, PositionWithdraw},
    swap, swap_claim, TradingPair,
};
use penumbra_proto::DomainType;
use penumbra_stake::{Delegate, Undelegate, UndelegateClaimBody};

use crate::{
    action::{
        DelegatorVote, DelegatorVoteBody, Proposal, ProposalDepositClaim, ProposalSubmit,
        ProposalWithdraw, ValidatorVote, ValidatorVoteBody, Vote,
    },
    plan::TransactionPlan,
    proposal, Action, Transaction, TransactionBody,
};

use penumbra_crypto::EffectingData as _;

// Note: temporarily duplicate of crypto/EffectingData
pub trait EffectingData {
    fn effect_hash(&self) -> EffectHash;
}

impl<'a, T: penumbra_crypto::EffectingData> EffectingData for crate::Compat<'a, T> {
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
    pub fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:tx_body")
            .to_state();

        // Hash the fixed data of the transaction body.
        state.update(chain_id_effect_hash(&self.chain_id).as_bytes());
        state.update(&self.expiry_height.to_le_bytes());
        state.update(self.fee.effect_hash().as_bytes());
        if self.memo.is_some() {
            let memo = self.memo.clone();
            state.update(&memo.unwrap().0);
        }

        // Hash the actions.
        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());
        for action in &self.actions {
            state.update(action.effect_hash().as_bytes());
        }

        // Hash the clues.
        let num_clues = self.fmd_clues.len() as u32;
        state.update(&num_clues.to_le_bytes());
        for fmd_clue in &self.fmd_clues {
            state.update(fmd_clue.effect_hash().as_bytes());
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
        state.update(chain_id_effect_hash(&self.chain_id).as_bytes());
        state.update(&self.expiry_height.to_le_bytes());
        state.update(self.fee.effect_hash().as_bytes());

        // Hash the memo and save the memo key for use with outputs later.
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self.memo_plan.clone().unwrap();
            state.update(memo_plan.memo().unwrap().0.as_ref());
            memo_key = Some(memo_plan.key);
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
        let num_clues = self.clue_plans.len() as u32;
        state.update(&num_clues.to_le_bytes());
        for clue_plan in self.clue_plans() {
            state.update(clue_plan.clue().effect_hash().as_bytes());
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

fn chain_id_effect_hash(chain_id: &str) -> Hash {
    blake2b_simd::Params::default()
        .personal(b"PAH:chain_id")
        .hash(chain_id.as_bytes())
}

impl EffectingData for Action {
    fn effect_hash(&self) -> EffectHash {
        match self {
            Action::Output(output) => crate::Compat(&output.body).effect_hash(),
            Action::Spend(spend) => crate::Compat(&spend.body).effect_hash(),
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

impl EffectingData for swap::Body {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:swap_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(self.trading_pair.effect_hash().as_bytes());
        state.update(&self.delta_1_i.to_le_bytes());
        state.update(&self.delta_2_i.to_le_bytes());
        state.update(&self.fee_commitment.to_bytes());
        state.update(&self.payload.commitment.0.to_bytes());
        state.update(&self.payload.encrypted_swap.0);

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for swap_claim::Body {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:swapclaimbdy")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.nullifier.0.to_bytes());
        state.update(self.fee.effect_hash().as_bytes());
        state.update(&self.output_1_commitment.0.to_bytes());
        state.update(&self.output_2_commitment.0.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Delegate {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:delegate")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.validator_identity.0.to_bytes());
        state.update(&self.epoch_index.to_le_bytes());
        state.update(&self.unbonded_amount.to_le_bytes());
        state.update(&self.delegation_amount.to_le_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Undelegate {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:undelegate")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.validator_identity.0.to_bytes());
        state.update(&self.start_epoch_index.to_le_bytes());
        state.update(&self.unbonded_amount.to_le_bytes());
        state.update(&self.delegation_amount.to_le_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for UndelegateClaimBody {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:udlgclm_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.validator_identity.0.to_bytes());
        state.update(&self.start_epoch_index.to_le_bytes());
        state.update(&self.penalty.0.to_le_bytes());
        state.update(&self.balance_commitment.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Proposal {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:proposal")
            .to_state();
        state.update(&self.encode_to_vec());
        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for ProposalSubmit {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_submit")
            .to_state();

        // These fields are all fixed-size
        state.update(&self.deposit_amount.to_le_bytes());

        // The proposal itself is variable-length, so we hash it, and then hash its hash in
        state.update(self.proposal.effect_hash().as_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for ProposalWithdraw {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_withdrw")
            .to_state();
        state.update(&self.encode_to_vec());
        EffectHash(state.finalize().as_array().clone())
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
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:vote")
            .to_state();

        state.update(match self {
            // Manually choose a distinct byte for each vote type
            Vote::Yes => b"Y",
            Vote::No => b"N",
            Vote::Abstain => b"A",
        });

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for ValidatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:val_vote")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&self.proposal.to_le_bytes());
        state.update(self.vote.effect_hash().as_bytes());
        state.update(&self.identity_key.0.to_bytes());
        state.update(&self.governance_key.0.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for DelegatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        let DelegatorVoteBody {
            proposal,
            start_position,
            vote,
            value,
            unbonded_amount,
            nullifier,
            rk,
        } = self;

        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:del_vote")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&proposal.to_le_bytes());
        state.update(&u64::from(*start_position).to_le_bytes());
        state.update(vote.effect_hash().as_bytes());
        state.update(&value.asset_id.0.to_bytes());
        state.update(&value.amount.to_le_bytes());
        state.update(&unbonded_amount.to_le_bytes());
        state.update(&nullifier.0.to_bytes());
        state.update(&rk.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for ProposalDepositClaim {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_dep_clm")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&self.proposal.to_le_bytes());
        state.update(self.outcome.effect_hash().as_bytes());
        state.update(&self.deposit_amount.to_le_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl<W> EffectingData for proposal::Outcome<W>
where
    proposal::Withdrawn<W>: EffectingData,
{
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_outcome")
            .to_state();

        match self {
            // Manually choose a distinct byte string prefix for each outcome type, all same length
            proposal::Outcome::Passed => {
                state.update(b"Passed");
            }
            proposal::Outcome::Failed { withdrawn } => {
                state.update(b"Failed");
                state.update(withdrawn.effect_hash().as_bytes());
            }
            proposal::Outcome::Slashed { withdrawn } => {
                state.update(b"Slashed");
                state.update(withdrawn.effect_hash().as_bytes());
            }
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for proposal::Withdrawn<String> {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:withdrawn")
            .to_state();

        match self {
            // Manually choose a distinct byte prefix for each case
            proposal::Withdrawn::No => {
                state.update(b"N");
            }
            proposal::Withdrawn::WithReason { reason } => {
                state.update(b"Y");
                state.update(reason.as_bytes());
            }
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for proposal::Withdrawn<()> {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:withdrawn_yn")
            .to_state();

        match self {
            // Manually choose a distinct byte prefix for each case
            proposal::Withdrawn::No => {
                state.update(b"N");
            }
            proposal::Withdrawn::WithReason { reason: () } => {
                state.update(b"Y");
            }
        }

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for PositionOpen {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_open")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&self.position.id().0);
        state.update(&self.position.reserves.r1.to_le_bytes());
        state.update(&self.position.reserves.r2.to_le_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for PositionClose {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_close")
            .to_state();

        state.update(&self.position_id.0);

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for PositionWithdraw {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_withdraw")
            .to_state();

        state.update(&self.position_id.0);
        state.update(&self.reserves_commitment.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for PositionRewardClaim {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_rewrdclm")
            .to_state();

        state.update(&self.position_id.0);
        state.update(&self.rewards_commitment.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for Clue {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:decaffmdclue")
            .to_state();

        state.update(&self.0);
        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for NotePayload {
    fn effect_hash(&self) -> EffectHash {
        EffectHash(
            blake2b_simd::Params::default()
                .personal(b"PAH:note_payload")
                .to_state()
                .update(&self.note_commitment.0.to_bytes())
                .update(&self.ephemeral_key.0)
                .update(&self.encrypted_note.0)
                .finalize()
                .as_array()
                .clone(),
        )
    }
}

impl EffectingData for Fee {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:fee")
            .to_state();
        state.update(&self.0.amount.to_le_bytes());
        state.update(&self.0.asset_id.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for TradingPair {
    fn effect_hash(&self) -> EffectHash {
        EffectHash(
            blake2b_simd::Params::default()
                .personal(b"PAH:trading_pair")
                .to_state()
                .update(&self.asset_1().to_bytes())
                .update(&self.asset_2().to_bytes())
                .finalize()
                .as_array()
                .clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use penumbra_crypto::{
        asset,
        keys::{SeedPhrase, SpendKey},
        memo::MemoPlaintext,
        Address, Fee, Note, Value, STAKING_TOKEN_ASSET_ID,
    };
    use penumbra_dex::{swap::SwapPlaintext, swap::SwapPlan, TradingPair};
    use penumbra_shielded_pool::{OutputPlan, SpendPlan};
    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::{
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
            penumbra_crypto::Value {
                amount: 10000u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );
        let note1 = Note::generate(
            &mut OsRng,
            &addr,
            penumbra_crypto::Value {
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
            .clone()
            .build(fvk, witness_data.clone())
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
