use ark_ff::UniformRand;
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use thiserror;

use crate::{
    action::{output, spend, Action},
    addresses::PaymentAddress,
    keys::{OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle,
    nullifier::Nullifier,
    transaction::{Transaction, TransactionBody},
    Fr, Note, Output, Spend, Value,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chain ID not set")]
    NoChainID,
    #[error("Expiry height not set")]
    ExpiryHeightNotSet,
    #[error("Fee not set")]
    FeeNotSet,
}

/// Used to construct a Penumbra transaction.
pub struct TransactionBuilder {
    // Actions we'll perform in this transaction.
    pub actions: Vec<Action>,
    // Transaction fee. None if unset.
    pub fee: Option<u64>,
    // Sum of blinding factors for each value commitment.
    pub synthetic_blinding_factor: Fr,
    // The root of the note commitment merkle tree.
    pub merkle_root: merkle::Root,
    // Expiry height. None if unset.
    pub expiry_height: Option<u32>,
    // Chain ID. None if unset.
    pub chain_id: Option<String>,
}

impl TransactionBuilder {
    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        spend_key: SpendKey,
        merkle_path: merkle::Path,
        note: Note,
    ) -> Self {
        // TODO: Derive nullifier from note commitment, note position, and
        // nullifier deriving key
        // See p.55 ZCash spec
        let nullifier = Nullifier::new();

        let v_blinding = Fr::rand(rng);
        let value_commitment = note.value.commit(v_blinding);
        // We add to the transaction's value balance.
        self.synthetic_blinding_factor += v_blinding;

        let spend_auth_randomizer = Fr::rand(rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let body = spend::Body::new(
            rng,
            value_commitment,
            nullifier,
            *spend_key.spend_auth_key(),
            spend_auth_randomizer,
            merkle_path,
        );

        let auth_sig = rsk.sign(rng, &body.serialize());

        let spend = Action::Spend(Spend { body, auth_sig });

        self.actions.push(spend);

        self
    }

    /// Create a new `Output` to create a new note.
    pub fn add_output<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        dest: &PaymentAddress,
        value_to_send: Value,
        memo: MemoPlaintext,
        _ovk: &OutgoingViewingKey,
    ) -> Self {
        let v_blinding = Fr::rand(rng);
        // We subtract from the transaction's value balance.
        self.synthetic_blinding_factor -= v_blinding;

        let body = output::Body::new(rng, value_to_send, v_blinding, dest);

        let encrypted_memo = memo.encrypt(dest);

        let ovk_wrapped_key = todo!();

        let output = Action::Output(Output {
            body,
            encrypted_memo,
            ovk_wrapped_key,
        });
        self.actions.push(output);

        self
    }

    /// Set the transaction fee in PEN.
    pub fn set_fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }

    /// Set the expiry height.
    pub fn set_expiry_height(mut self, expiry_height: u32) -> Self {
        self.expiry_height = Some(expiry_height);
        self
    }

    /// Set the chain ID.
    pub fn set_chain_id(mut self, chain_id: String) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    pub fn finalize<R: CryptoRng + RngCore>(mut self, rng: &mut R) -> Result<Transaction, Error> {
        // Randomize all actions (including outputs) to minimize info leakage.
        self.actions.shuffle(rng);

        if self.chain_id.is_none() {
            return Err(Error::NoChainID);
        }

        if self.expiry_height.is_none() {
            return Err(Error::ExpiryHeightNotSet);
        }

        if self.fee.is_none() {
            return Err(Error::FeeNotSet);
        }

        let _transaction_body = TransactionBody {
            merkle_root: self.merkle_root,
            actions: self.actions,
            expiry_height: self.expiry_height.unwrap(),
            chain_id: self.chain_id.unwrap(),
            fee: self.fee.unwrap(),
        };

        // Apply sig
        todo!();
    }
}
