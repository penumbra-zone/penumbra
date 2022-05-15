use blake2b_simd::{Hash, Params};
use decaf377::FieldExt;
use penumbra_crypto::FullViewingKey;
use penumbra_proto::{transaction as pb, Message, Protobuf};

use crate::{
    action::{output, spend, Delegate, Undelegate},
    plan::{ActionPlan, TransactionPlan},
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
        // don't need to actually construct the `TransactionBody`.  We delegate
        // to `ActionPlan::auth_hash` to avoid having to construct complete
        // actions.

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
            state.update(action.auth_hash(fvk).as_bytes());
        }

        AuthHash(*state.finalize().as_array())
    }
}

fn chain_id_auth_hash(chain_id: &str) -> Hash {
    blake2b_simd::Params::default()
        .personal(b"PAH:chain_id")
        .hash(&chain_id.as_bytes())
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

impl ActionPlan {
    fn auth_hash(&self, fvk: &FullViewingKey) -> Hash {
        match self {
            ActionPlan::Output(output) => output.output_body(fvk.outgoing()).auth_hash(),
            ActionPlan::Spend(spend) => spend.spend_body(fvk).auth_hash(),
            ActionPlan::Delegate(delegate) => delegate.auth_hash(),
            ActionPlan::Undelegate(undelegate) => undelegate.auth_hash(),
            // These are data payloads, so just hash them directly,
            // since we consider them authorizing data.
            ActionPlan::ValidatorDefinition(payload) => Params::default()
                .personal(b"PAH:valdefnition")
                .hash(&payload.encode_to_vec()),
            ActionPlan::IBCAction(payload) => Params::default()
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
