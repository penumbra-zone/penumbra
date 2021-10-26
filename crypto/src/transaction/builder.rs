use ark_ff::UniformRand;
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use std::ops::Deref;
use thiserror;

use crate::rdsa::{Binding, Signature, SigningKey};
use crate::{
    action::constants::OVK_WRAPPED_LEN_BYTES,
    action::{output, spend, Action},
    keys::{OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle,
    transaction::{Fee, Transaction, TransactionBody},
    value, Address, Fq, Fr, Note, Output, Spend, Value,
};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Chain ID not set")]
    NoChainID,
    #[error("Fee not set")]
    FeeNotSet,
    #[error("Value balance of this transaction is not zero")]
    NonZeroValueBalance,
}

/// Used to construct a Penumbra transaction.
pub struct Builder {
    // Actions we'll perform in this transaction.
    pub actions: Vec<Action>,
    // Transaction fee. None if unset.
    pub fee: Option<Fee>,
    // Sum of blinding factors for each value commitment.
    pub synthetic_blinding_factor: Fr,
    // Sum of value commitments.
    pub value_commitments: decaf377::Element,
    // Value balance.
    pub value_balance: decaf377::Element,
    // The root of the note commitment merkle tree.
    pub merkle_root: merkle::Root,
    // Expiry height. None if unset.
    pub expiry_height: Option<u32>,
    // Chain ID. None if unset.
    pub chain_id: Option<String>,
}

impl Builder {
    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        spend_key: SpendKey,
        merkle_path: merkle::Path,
        note: Note,
        position: merkle::Position,
    ) -> Self {
        let v_blinding = Fr::rand(rng);
        let value_commitment = note.value().commit(v_blinding);
        // We add to the transaction's value balance.
        self.synthetic_blinding_factor += v_blinding;
        self.value_balance +=
            Fr::from(note.value().amount) * note.value().asset_id.value_generator();

        let spend_auth_randomizer = Fr::rand(rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let body = spend::Body::new(
            rng,
            value_commitment,
            *spend_key.spend_auth_key(),
            spend_auth_randomizer,
            merkle_path,
            position,
            note,
            v_blinding,
            *spend_key.nullifier_key(),
        );
        self.value_commitments += value_commitment.0;

        let body_serialized: Vec<u8> = body.clone().into();
        let auth_sig = rsk.sign(rng, &body_serialized);

        let spend = Action::Spend(Spend { body, auth_sig });

        self.actions.push(spend);

        self
    }

    /// Create a new `Output` to create a new note.
    pub fn add_output<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        dest: &Address,
        value_to_send: Value,
        memo: MemoPlaintext,
        _ovk: &OutgoingViewingKey,
    ) -> Self {
        let v_blinding = Fr::rand(rng);
        // We subtract from the transaction's value balance.
        self.synthetic_blinding_factor -= v_blinding;
        self.value_balance -=
            Fr::from(value_to_send.amount) * value_to_send.asset_id.value_generator();

        let note_blinding = Fq::rand(rng);

        let note = Note::new(
            *dest.diversifier(),
            dest.transmission_key(),
            value_to_send,
            note_blinding,
        )
        .expect("transmission key is valid");
        let body = output::Body::new(rng, note, v_blinding, dest);
        self.value_commitments -= body.value_commitment.0;

        // TODO!
        let encrypted_memo = memo.encrypt(dest);

        // TODO!
        let ovk_wrapped_key = [0u8; OVK_WRAPPED_LEN_BYTES];

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
        self.fee = Some(Fee(fee));
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

    /// Add the binding signature based on the current sum of synthetic blinding factors.
    #[allow(non_snake_case)]
    pub fn compute_binding_sig<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        transaction_body: TransactionBody,
    ) -> Signature<Binding> {
        let binding_signing_key: SigningKey<Binding> = self.synthetic_blinding_factor.into();

        // Check that the derived verification key corresponds to the signing key to be used.
        let H = value::VALUE_BLINDING_GENERATOR.deref();
        let binding_verification_key_raw = (self.synthetic_blinding_factor * H).compress().0;

        // If value balance is non-zero, the verification key would be value_commitments - value_balance,
        // but value_balance should always be zero.
        let computed_verification_key = self.value_commitments.compress().0;
        assert_eq!(binding_verification_key_raw, computed_verification_key);

        let transaction_body_serialized: Vec<u8> = transaction_body.into();
        binding_signing_key.sign(rng, &transaction_body_serialized)
    }

    pub fn finalize<R: CryptoRng + RngCore>(mut self, rng: &mut R) -> Result<Transaction, Error> {
        // Randomize all actions (including outputs) to minimize info leakage.
        self.actions.shuffle(rng);

        if self.chain_id.is_none() {
            return Err(Error::NoChainID);
        }

        if self.fee.is_none() {
            return Err(Error::FeeNotSet);
        }

        if self.value_balance != decaf377::Element::default() {
            return Err(Error::NonZeroValueBalance);
        }

        let transaction_body = TransactionBody {
            merkle_root: self.merkle_root.clone(),
            actions: self.actions.clone(),
            expiry_height: self.expiry_height.unwrap_or(0),
            chain_id: self.chain_id.clone().unwrap(),
            fee: self.fee.clone().unwrap(),
        };

        let binding_sig = self.compute_binding_sig(rng, transaction_body.clone());

        Ok(Transaction {
            transaction_body,
            binding_sig,
        })
    }
}
