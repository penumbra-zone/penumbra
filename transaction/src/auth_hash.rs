use blake2b_simd::{Hash, Params};
use decaf377::FieldExt;
use decaf377_fmd::Clue;
use penumbra_crypto::{FullViewingKey, PayloadKey};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Message, Protobuf};

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

pub trait AuthorizingData {
    fn auth_hash(&self) -> Hash;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AuthHash([u8; 64]);

impl Default for AuthHash {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl std::fmt::Debug for AuthHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AuthHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl Protobuf<pb::AuthHash> for AuthHash {}

impl From<AuthHash> for pb::AuthHash {
    fn from(msg: AuthHash) -> Self {
        Self {
            inner: msg.0.to_vec().into(),
        }
    }
}

impl TryFrom<pb::AuthHash> for AuthHash {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthHash) -> Result<Self, Self::Error> {
        Ok(Self(value.inner.as_ref().try_into()?))
    }
}

impl AsRef<[u8]> for AuthHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Transaction {
    pub fn auth_hash(&self) -> AuthHash {
        self.transaction_body.auth_hash()
    }
}

impl TransactionBody {
    pub fn auth_hash(&self) -> AuthHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:tx_body")
            .to_state();

        // Hash the fixed data of the transaction body.
        state.update(chain_id_auth_hash(&self.chain_id).as_bytes());
        state.update(&self.expiry_height.to_le_bytes());
        state.update(self.fee.auth_hash().as_bytes());
        if self.memo.is_some() {
            let memo = self.memo.clone();
            state.update(&memo.unwrap().0);
        }

        // Hash the actions.
        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());
        for action in &self.actions {
            state.update(action.auth_hash().as_bytes());
        }

        // Hash the clues.
        let num_clues = self.fmd_clues.len() as u32;
        state.update(&num_clues.to_le_bytes());
        for fmd_clue in &self.fmd_clues {
            state.update(fmd_clue.auth_hash().as_bytes());
        }

        AuthHash(*state.finalize().as_array())
    }
}

impl TransactionPlan {
    /// Computes the [`AuthHash`] for the [`Transaction`] described by this
    /// [`TransactionPlan`].
    ///
    /// This method does not require constructing the entire [`Transaction`],
    /// but it does require the associated [`FullViewingKey`] to derive
    /// authorizing data that will be fed into the hash.
    pub fn auth_hash(&self, fvk: &FullViewingKey) -> AuthHash {
        // This implementation is identical to the one above, except that we
        // don't need to actually construct the entire `TransactionBody` with
        // complete `Action`s, we just need to construct the bodies of the
        // actions the transaction will have when constructed.

        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:tx_body")
            .to_state();

        // Hash the fixed data of the transaction body.
        state.update(chain_id_auth_hash(&self.chain_id).as_bytes());
        state.update(&self.expiry_height.to_le_bytes());
        state.update(self.fee.auth_hash().as_bytes());

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
            state.update(spend.spend_body(fvk).auth_hash().as_bytes());
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
                    .auth_hash()
                    .as_bytes(),
            );
        }
        for swap in self.swap_plans() {
            state.update(swap.swap_body(fvk).auth_hash().as_bytes());
        }
        for swap_claim in self.swap_claim_plans() {
            state.update(swap_claim.swap_claim_body(fvk).auth_hash().as_bytes());
        }
        for delegation in self.delegations() {
            state.update(delegation.auth_hash().as_bytes());
        }
        for undelegation in self.undelegations() {
            state.update(undelegation.auth_hash().as_bytes());
        }
        for undelegation_claim in self.undelegate_claim_plans() {
            // TODO: build from plan
            state.update(undelegation_claim.auth_hash().as_bytes());
        }
        for proposal_submit in self.proposal_submits() {
            state.update(proposal_submit.auth_hash().as_bytes());
        }
        for proposal_withdraw in self.proposal_withdraws() {
            state.update(proposal_withdraw.auth_hash().as_bytes());
        }
        for validator_vote in self.validator_votes() {
            state.update(validator_vote.auth_hash().as_bytes());
        }
        for _delegator_vote in self.delegator_vote_plans() {
            // TODO: get the authorization hash of the delegator vote body for each plan
        }
        // These are data payloads, so just hash them directly,
        // since we consider them authorizing data.
        for payload in self.validator_definitions() {
            let auth_hash = Params::default()
                .personal(b"PAH:valdefnition")
                .hash(&payload.encode_to_vec());
            state.update(auth_hash.as_bytes());
        }
        for payload in self.ibc_actions() {
            let auth_hash = Params::default()
                .personal(b"PAH:ibc_action")
                .hash(&payload.encode_to_vec());
            state.update(auth_hash.as_bytes());
        }
        let num_clues = self.clue_plans.len() as u32;
        state.update(&num_clues.to_le_bytes());
        for clue_plan in self.clue_plans() {
            state.update(clue_plan.clue().auth_hash().as_bytes());
        }

        AuthHash(*state.finalize().as_array())
    }
}

fn chain_id_auth_hash(chain_id: &str) -> Hash {
    blake2b_simd::Params::default()
        .personal(b"PAH:chain_id")
        .hash(chain_id.as_bytes())
}

impl AuthorizingData for Action {
    fn auth_hash(&self) -> Hash {
        match self {
            Action::Output(output) => output.body.auth_hash(),
            Action::Spend(spend) => spend.body.auth_hash(),
            Action::Delegate(delegate) => delegate.auth_hash(),
            Action::Undelegate(undelegate) => undelegate.auth_hash(),
            Action::UndelegateClaim(claim) => claim.body.auth_hash(),
            Action::ProposalSubmit(submit) => submit.auth_hash(),
            Action::ProposalWithdraw(withdraw) => withdraw.auth_hash(),
            Action::ValidatorVote(vote) => vote.auth_hash(),
            Action::SwapClaim(swap_claim) => swap_claim.body.auth_hash(),
            Action::Swap(swap) => swap.body.auth_hash(),
            // These are data payloads, so just hash them directly,
            // since we consider them authorizing data.
            Action::ValidatorDefinition(payload) => Params::default()
                .personal(b"PAH:valdefnition")
                .hash(&payload.encode_to_vec()),
            Action::IBCAction(payload) => Params::default()
                .personal(b"PAH:ibc_action")
                .hash(&payload.encode_to_vec()),

            Action::PositionOpen(p) => p.auth_hash(),
            Action::PositionClose(p) => p.auth_hash(),
            Action::PositionWithdraw(p) => p.auth_hash(),
            Action::PositionRewardClaim(p) => p.auth_hash(),
            Action::Ics20Withdrawal(w) => w.auth_hash(),
        }
    }
}

impl AuthorizingData for output::Body {
    fn auth_hash(&self) -> Hash {
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

        state.finalize()
    }
}

impl AuthorizingData for spend::Body {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:spend_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.balance_commitment.to_bytes());
        state.update(&self.nullifier.0.to_bytes());
        state.update(&self.rk.to_bytes());

        state.finalize()
    }
}

impl AuthorizingData for swap::Body {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:swap_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(self.trading_pair.auth_hash().as_bytes());
        state.update(&self.delta_1_i.to_le_bytes());
        state.update(&self.delta_2_i.to_le_bytes());
        state.update(&self.fee_commitment.to_bytes());
        state.update(self.swap_nft.auth_hash().as_bytes());
        state.update(&self.swap_ciphertext.0);

        state.finalize()
    }
}

impl AuthorizingData for swap_claim::Body {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:swapclaimbdy")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.nullifier.0.to_bytes());
        state.update(self.fee.auth_hash().as_bytes());
        state.update(self.output_1.auth_hash().as_bytes());
        state.update(self.output_2.auth_hash().as_bytes());
        state.update(self.output_data.auth_hash().as_bytes());

        state.finalize()
    }
}

impl AuthorizingData for Delegate {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:delegate")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.validator_identity.0.to_bytes());
        state.update(&self.epoch_index.to_le_bytes());
        state.update(&self.unbonded_amount.to_le_bytes());
        state.update(&self.delegation_amount.to_le_bytes());

        state.finalize()
    }
}

impl AuthorizingData for Undelegate {
    fn auth_hash(&self) -> Hash {
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

        state.finalize()
    }
}

impl AuthorizingData for UndelegateClaimBody {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:udlgclm_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.validator_identity.0.to_bytes());
        state.update(&self.start_epoch_index.to_le_bytes());
        state.update(&self.end_epoch_index.to_le_bytes());
        state.update(&self.penalty.to_le_bytes());
        state.update(&self.balance_commitment.to_bytes());

        state.finalize()
    }
}

impl AuthorizingData for Proposal {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:proposal")
            .to_state();
        state.update(&self.encode_to_vec());
        state.finalize()
    }
}

impl AuthorizingData for ProposalSubmit {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_submit")
            .to_state();

        // These fields are all fixed-size
        state.update(&self.deposit_amount.to_le_bytes());
        // The address is hashed as a string, which is the canonical bech32 encoding of the address
        state.update(self.deposit_refund_address.to_string().as_bytes());
        state.update(&self.withdraw_proposal_key.to_bytes());

        // The proposal itself is variable-length, so we hash it, and then hash its hash in
        state.update(self.proposal.auth_hash().as_bytes());

        state.finalize()
    }
}

impl AuthorizingData for ProposalWithdraw {
    fn auth_hash(&self) -> Hash {
        self.body.auth_hash()
    }
}

impl AuthorizingData for ProposalWithdrawPlan {
    fn auth_hash(&self) -> Hash {
        self.body.auth_hash()
    }
}

impl AuthorizingData for ProposalWithdrawBody {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:prop_withdrw")
            .to_state();
        state.update(&self.encode_to_vec());
        state.finalize()
    }
}

impl AuthorizingData for ValidatorVote {
    fn auth_hash(&self) -> Hash {
        self.body.auth_hash()
    }
}

impl AuthorizingData for Vote {
    fn auth_hash(&self) -> Hash {
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

        state.finalize()
    }
}

impl AuthorizingData for ValidatorVoteBody {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:val_vote")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&self.proposal.to_le_bytes());
        state.update(self.vote.auth_hash().as_bytes());
        state.update(&self.identity_key.0.to_bytes());

        state.finalize()
    }
}

impl AuthorizingData for PositionOpen {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_open")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them in the hash one after the
        // other.
        state.update(&self.position.id().0);
        state.update(&self.initial_reserves.r1.to_le_bytes());
        state.update(&self.initial_reserves.r2.to_le_bytes());

        state.finalize()
    }
}

impl AuthorizingData for PositionClose {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_close")
            .to_state();

        state.update(&self.position_id.0);

        state.finalize()
    }
}

impl AuthorizingData for PositionWithdraw {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_withdraw")
            .to_state();

        state.update(&self.position_id.0);
        state.update(&self.reserves_commitment.to_bytes());

        state.finalize()
    }
}

impl AuthorizingData for PositionRewardClaim {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:pos_rewrdclm")
            .to_state();

        state.update(&self.position_id.0);
        state.update(&self.rewards_commitment.to_bytes());

        state.finalize()
    }
}

impl AuthorizingData for Ics20Withdrawal {
    fn auth_hash(&self) -> Hash {
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
        state.finalize()
    }
}

impl AuthorizingData for Clue {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:decaffmdclue")
            .to_state();

        state.update(&self.0);
        state.finalize()
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
    fn plan_auth_hash_matches_transaction_auth_hash() {
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

        let plan_auth_hash = plan.auth_hash(fvk);

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

        let transaction_auth_hash = transaction.auth_hash();

        assert_eq!(plan_auth_hash, transaction_auth_hash);
    }
}
