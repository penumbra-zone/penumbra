use blake2b_simd::{Hash, Params};
use decaf377::FieldExt;
use decaf377_fmd::Clue;
use penumbra_crypto::{
    dex::TradingPair, transaction::Fee, EncryptedNote, FullViewingKey, PayloadKey,
};
use penumbra_proto::{core::crypto::v1alpha1 as pb_crypto, Message, Protobuf};

use crate::{
    action::{
        output, spend, swap, swap_claim, Delegate, Ics20Withdrawal, PositionClose, PositionOpen,
        PositionRewardClaim, PositionWithdraw, Proposal, ProposalSubmit, ProposalWithdraw,
        ProposalWithdrawBody, Undelegate, UndelegateClaimBody, ValidatorVote, ValidatorVoteBody,
        Vote,
    },
    plan::{ProposalWithdrawPlan, TransactionPlan},
    Action, Transaction, TransactionBody,
};

pub trait EffectingData {
    fn effect_hash(&self) -> EffectHash;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EffectHash([u8; 64]);

impl EffectHash {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Default for EffectHash {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl std::fmt::Debug for EffectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EffectHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl Protobuf<pb_crypto::EffectHash> for EffectHash {}

impl From<EffectHash> for pb_crypto::EffectHash {
    fn from(msg: EffectHash) -> Self {
        Self {
            inner: msg.0.to_vec().into(),
        }
    }
}

impl TryFrom<pb_crypto::EffectHash> for EffectHash {
    type Error = anyhow::Error;
    fn try_from(value: pb_crypto::EffectHash) -> Result<Self, Self::Error> {
        Ok(Self(value.inner.try_into().map_err(|_| {
            anyhow::anyhow!("incorrect length for effect hash")
        })?))
    }
}

impl AsRef<[u8]> for EffectHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
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
        for _delegator_vote in self.delegator_vote_plans() {
            // TODO: get the effecthash of the delegator vote body for each plan
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
            Action::Output(output) => output.body.effect_hash(),
            Action::Spend(spend) => spend.body.effect_hash(),
            Action::Delegate(delegate) => delegate.effect_hash(),
            Action::Undelegate(undelegate) => undelegate.effect_hash(),
            Action::UndelegateClaim(claim) => claim.body.effect_hash(),
            Action::ProposalSubmit(submit) => submit.effect_hash(),
            Action::ProposalWithdraw(withdraw) => withdraw.effect_hash(),
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
            Action::IBCAction(payload) => EffectHash(
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
        }
    }
}

impl EffectingData for output::Body {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:output_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.note_payload.note_commitment.0.to_bytes());
        state.update(&self.note_payload.ephemeral_key.0);
        state.update(&self.note_payload.encrypted_note);
        state.update(&self.balance_commitment.to_bytes());
        state.update(&self.wrapped_memo_key.0);
        state.update(&self.ovk_wrapped_key.0);

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for spend::Body {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:spend_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.balance_commitment.to_bytes());
        state.update(&self.nullifier.0.to_bytes());
        state.update(&self.rk.to_bytes());

        EffectHash(state.finalize().as_array().clone())
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
        state.update(&self.payload.ephemeral_key.0);
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
        state.update(&self.end_epoch_index.to_le_bytes());
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
        state.update(&self.end_epoch_index.to_le_bytes());
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
        // The address is hashed as a string, which is the canonical bech32 encoding of the address
        state.update(self.deposit_refund_address.to_string().as_bytes());
        state.update(&self.withdraw_proposal_key.to_bytes());

        // The proposal itself is variable-length, so we hash it, and then hash its hash in
        state.update(self.proposal.effect_hash().as_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl EffectingData for ProposalWithdraw {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
    }
}

impl EffectingData for ProposalWithdrawPlan {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
    }
}

impl EffectingData for ProposalWithdrawBody {
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
            Vote::NoWithVeto => b"V",
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
        state.update(&self.initial_reserves.r1.to_le_bytes());
        state.update(&self.initial_reserves.r2.to_le_bytes());

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

impl EffectingData for Ics20Withdrawal {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:ics20wthdrwl")
            .to_state();

        let destination_chain_id_hash =
            blake2b_simd::Params::default().hash(self.destination_chain_id.as_bytes());
        let destination_chain_address_hash =
            blake2b_simd::Params::default().hash(self.destination_chain_address.as_bytes());

        state.update(destination_chain_id_hash.as_bytes());
        state.update(&self.value().amount.to_le_bytes());
        state.update(&self.value().asset_id.to_bytes());
        state.update(destination_chain_address_hash.as_bytes());
        //This is safe because the return address has a constant length of 80 bytes.
        state.update(&self.return_address.to_vec());
        state.update(&self.timeout_height.to_le_bytes());
        state.update(&self.timeout_time.to_le_bytes());
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

impl EffectingData for EncryptedNote {
    fn effect_hash(&self) -> EffectHash {
        EffectHash(
            blake2b_simd::Params::default()
                .personal(b"PAH:note_payload")
                .to_state()
                .update(&self.note_commitment.0.to_bytes())
                .update(&self.ephemeral_key.0)
                .update(&self.encrypted_note)
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
        dex::{swap::SwapPlaintext, TradingPair},
        keys::{SeedPhrase, SpendKey},
        transaction::Fee,
        Note, Value, STAKING_TOKEN_ASSET_ID,
    };
    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::{
        plan::{CluePlan, MemoPlan, OutputPlan, SpendPlan, SwapPlan, TransactionPlan},
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
        let (addr, _dtk) = fvk.incoming().payment_address(0u64.into());

        let mut nct = tct::Tree::new();

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

        nct.insert(tct::Witness::Keep, note0.commit()).unwrap();
        nct.insert(tct::Witness::Keep, note1.commit()).unwrap();

        let trading_pair = TradingPair::new(
            asset::REGISTRY.parse_denom("nala").unwrap().id(),
            asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        )
        .unwrap();

        let swap_plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            100000u64.into(),
            1u64.into(),
            Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            }),
            addr,
        );

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
            memo_plan: Some(MemoPlan::new(&mut OsRng, String::new()).unwrap()),
        };

        println!("{}", serde_json::to_string_pretty(&plan).unwrap());

        let plan_effect_hash = plan.effect_hash(fvk);

        let auth_data = plan.authorize(rng, &sk);
        let witness_data = WitnessData {
            anchor: nct.root(),
            note_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    (
                        spend.note.commit(),
                        nct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let transaction = plan
            .build(&mut OsRng, fvk, auth_data, witness_data)
            .unwrap();

        let transaction_effect_hash = transaction.effect_hash();

        assert_eq!(plan_effect_hash, transaction_effect_hash);
    }
}
