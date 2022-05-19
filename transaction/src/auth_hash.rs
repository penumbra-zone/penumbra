use blake2b_simd::{Hash, Params};
use decaf377::FieldExt;
use penumbra_crypto::FullViewingKey;
use penumbra_proto::{transaction as pb, Message, Protobuf};

use crate::{
    action::{output, spend, Delegate, Undelegate},
    plan::TransactionPlan,
    Action, Fee, Transaction, TransactionBody,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AuthHash([u8; 64]);

impl std::fmt::Debug for AuthHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AuthHash")
            .field(&hex::encode(&self.0))
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

        // Hash the actions.
        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());
        for action in &self.actions {
            state.update(action.auth_hash().as_bytes());
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

        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());

        // TransactionPlan::build builds the actions sorted by type, so hash the
        // actions in the order they'll appear in the final transaction.
        for spend in self.spend_plans() {
            state.update(spend.spend_body(fvk).auth_hash().as_bytes());
        }
        for output in self.output_plans() {
            state.update(output.output_body(fvk.outgoing()).auth_hash().as_bytes());
        }
        for delegation in self.delegations() {
            state.update(delegation.auth_hash().as_bytes());
        }
        for undelegation in self.undelegations() {
            state.update(undelegation.auth_hash().as_bytes());
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

        AuthHash(*state.finalize().as_array())
    }
}

fn chain_id_auth_hash(chain_id: &str) -> Hash {
    blake2b_simd::Params::default()
        .personal(b"PAH:chain_id")
        .hash(chain_id.as_bytes())
}

impl Fee {
    fn auth_hash(&self) -> Hash {
        blake2b_simd::Params::default()
            .personal(b"PAH:fee")
            .hash(&self.0.to_le_bytes())
    }
}

impl Action {
    fn auth_hash(&self) -> Hash {
        match self {
            Action::Output(output) => output.body.auth_hash(),
            Action::Spend(spend) => spend.body.auth_hash(),
            Action::Delegate(delegate) => delegate.auth_hash(),
            Action::Undelegate(undelegate) => undelegate.auth_hash(),
            // These are data payloads, so just hash them directly,
            // since we consider them authorizing data.
            Action::ValidatorDefinition(payload) => Params::default()
                .personal(b"PAH:valdefnition")
                .hash(&payload.encode_to_vec()),
            Action::IBCAction(payload) => Params::default()
                .personal(b"PAH:ibc_action")
                .hash(&payload.encode_to_vec()),
        }
    }
}

impl output::Body {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:output_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.note_payload.note_commitment.0.to_bytes());
        state.update(&self.note_payload.ephemeral_key.0);
        state.update(&self.note_payload.encrypted_note);
        state.update(&self.value_commitment.to_bytes());
        state.update(&self.encrypted_memo.0);
        state.update(&self.ovk_wrapped_key);

        state.finalize()
    }
}

impl spend::Body {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:spend_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.value_commitment.to_bytes());
        state.update(&self.nullifier.0.to_bytes());
        state.update(&self.rk.to_bytes());

        state.finalize()
    }
}

impl Delegate {
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

impl Undelegate {
    fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:undelegate")
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

#[cfg(test)]
mod tests {
    use incrementalmerkletree::{Frontier, Tree};
    use penumbra_crypto::{
        keys::{SeedPhrase, SpendKey, SpendSeed},
        memo::MemoPlaintext,
        merkle::NoteCommitmentTree,
        merkle::TreeExt,
        Note, Value, STAKING_TOKEN_ASSET_ID,
    };
    use rand_core::OsRng;

    use crate::{
        plan::{OutputPlan, SpendPlan, TransactionPlan},
        Fee, WitnessData,
    };

    /// This isn't an exhaustive test, but we don't currently have a
    /// great way to generate actions for randomized testing.
    ///
    /// All we hope to check here is that, for a basic transaction plan,
    /// we compute the same auth hash for the plan and for the transaction.
    #[test]
    fn plan_auth_hash_matches_transaction_auth_hash() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let sk = SpendKey::new(spend_seed);
        let fvk = sk.full_viewing_key();
        let (addr, _dtk) = fvk.incoming().payment_address(0u64.into());

        let mut nct = NoteCommitmentTree::new(0);

        let note0 = Note::generate(
            &mut OsRng,
            &addr,
            penumbra_crypto::Value {
                amount: 10000,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );
        let note1 = Note::generate(
            &mut OsRng,
            &addr,
            penumbra_crypto::Value {
                amount: 20000,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );

        nct.append(&note0.commit());
        nct.witness();
        nct.append(&note1.commit());
        nct.witness();

        let plan = TransactionPlan {
            expiry_height: 0,
            fee: Fee(0),
            chain_id: "penumbra-test".to_string(),
            // Put outputs first to check that the auth hash
            // computation is not affected by plan ordering.
            actions: vec![
                OutputPlan::new(
                    &mut OsRng,
                    Value {
                        amount: 30000,
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    addr.clone(),
                    MemoPlaintext::default(),
                )
                .into(),
                SpendPlan::new(&mut OsRng, note0, 0usize.into()).into(),
                SpendPlan::new(&mut OsRng, note1, 1usize.into()).into(),
            ],
        };

        println!("{}", serde_json::to_string_pretty(&plan).unwrap());

        let plan_auth_hash = plan.auth_hash(fvk);

        let auth_data = plan.authorize(rng, &sk);
        let witness_data = WitnessData {
            anchor: nct.root2(),
            auth_paths: plan
                .spend_plans()
                .map(|spend| nct.auth_path(spend.note.commit()).unwrap())
                .collect(),
        };
        let transaction = plan
            .build(&mut OsRng, fvk, auth_data, witness_data)
            .unwrap();

        let transaction_auth_hash = transaction.auth_hash();

        assert_eq!(plan_auth_hash, transaction_auth_hash);
    }
}
