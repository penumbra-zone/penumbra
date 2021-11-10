use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};
use std::ops::Deref;

use super::Error;
use crate::note::OVK_WRAPPED_LEN_BYTES;
use crate::rdsa::{Binding, Signature, SigningKey};
use crate::{
    action::{output, Action},
    ka,
    memo::{MemoCiphertext, MEMO_CIPHERTEXT_LEN_BYTES},
    merkle,
    transaction::{Fee, Transaction, TransactionBody},
    value, Fr, Note, Output,
};

/// Used to construct a Penumbra transaction from genesis notes.
///
/// When genesis notes are created, we construct a single genesis transaction
/// for them such that all transactions (genesis and non-genesis) can be
/// treated equally by clients.
///
/// The `GenesisBuilder` has no way to create spends, only outputs, and
/// allows for a non-zero value balance.
pub struct GenesisBuilder {
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

impl GenesisBuilder {
    /// Create a new `Output` for the genesis note.
    pub fn add_output<R: RngCore + CryptoRng>(&mut self, rng: &mut R, note: Note) {
        let v_blinding = Fr::rand(rng);
        // We subtract from the transaction's value balance.
        self.synthetic_blinding_factor -= v_blinding;
        self.value_balance -= Fr::from(note.amount()) * note.asset_id().value_generator();

        let esk = ka::Secret::new(rng);
        let body = output::Body::new(
            note.clone(),
            v_blinding,
            note.diversified_generator(),
            note.transmission_key(),
            &esk,
        );
        self.value_commitments -= body.value_commitment.0;

        // xx Hardcore something in the memo for genesis?
        // let encrypted_memo = memo.encrypt(&esk, &dest);
        let encrypted_memo = MemoCiphertext([0u8; MEMO_CIPHERTEXT_LEN_BYTES]);

        // In the case of genesis notes, the notes are transparent, so we fill
        // the `ovk_wrapped_key` field with 0s.
        let ovk_wrapped_key = [0u8; OVK_WRAPPED_LEN_BYTES];

        let output = Action::Output(Output {
            body,
            encrypted_memo,
            ovk_wrapped_key,
        });
        self.actions.push(output);
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

        // For the genesis transaction there will be a non-zero value balance since we are creating value.
        let computed_verification_key = (self.value_commitments - self.value_balance).compress().0;
        assert_eq!(binding_verification_key_raw, computed_verification_key);

        let transaction_body_serialized: Vec<u8> = transaction_body.into();
        binding_signing_key.sign(rng, &transaction_body_serialized)
    }

    pub fn finalize<R: CryptoRng + RngCore>(self, rng: &mut R) -> Result<Transaction, Error> {
        if self.chain_id.is_none() {
            return Err(Error::NoChainID);
        }

        let transaction_body = TransactionBody {
            merkle_root: self.merkle_root.clone(),
            actions: self.actions.clone(),
            expiry_height: 0,
            chain_id: self.chain_id.clone().unwrap(),
            fee: Fee(0),
        };

        let binding_sig = self.compute_binding_sig(rng, transaction_body.clone());

        Ok(Transaction {
            transaction_body,
            binding_sig,
        })
    }
}
