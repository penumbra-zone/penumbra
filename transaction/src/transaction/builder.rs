use std::ops::Deref;

use ark_ff::{UniformRand, Zero};
use incrementalmerkletree::Tree;
use penumbra_crypto::{
    keys::{OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle::{self, NoteCommitmentTree},
    proofs::transparent::SpendProof,
    rdsa::{Binding, Signature, SigningKey, SpendAuth},
    value, Address, Fr, IdentityKey, Note, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{stake as pbs, Message};
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};

use crate::{
    action::{spend, Action, Delegate, Output, Spend, Undelegate},
    AuthHash, Error, Fee, Transaction, TransactionBody,
};

/// Used to construct a Penumbra transaction.
pub struct Builder {
    /// List of spends. We store the spend key and body rather than a Spend
    /// so we can defer signing until the complete transaction is ready.
    pub spends: Vec<(SigningKey<SpendAuth>, SpendProof, spend::Body)>,
    /// List of outputs in the transaction.
    pub outputs: Vec<Output>,
    /// List of delegations in the transaction.
    pub delegations: Vec<Delegate>,
    /// List of undelegations in the transaction.
    pub undelegations: Vec<Undelegate>,
    /// List of validator (re-)definitions in the transaction.
    pub validator_definitions: Vec<pbs::ValidatorDefinition>,
    /// Transaction fee. None if unset.
    pub fee: Option<Fee>,
    /// Sum of blinding factors for each value commitment.
    pub synthetic_blinding_factor: Fr,
    /// Sum of value commitments.
    pub value_commitments: decaf377::Element,
    /// Value balance.
    pub value_balance: decaf377::Element,
    /// The root of the note commitment merkle tree.
    pub merkle_root: merkle::Root,
    /// Expiry height. None if unset.
    pub expiry_height: Option<u64>,
    /// Chain ID. None if unset.
    pub chain_id: Option<String>,
}

impl Builder {
    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        note_commitment_tree: &NoteCommitmentTree,
        spend_key: &SpendKey,
        note: Note,
    ) -> Result<&mut Self, anyhow::Error> {
        let merkle_path = note_commitment_tree
            .authentication_path(&note.commit())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Note commitment tree cannot construct an auth path for note commitment {:?}",
                    note.commit()
                )
            })?;

        let v_blinding = Fr::rand(rng);
        let value_commitment = note.value().commit(v_blinding);

        // Spends add to the transaction's value balance.
        self.synthetic_blinding_factor += v_blinding;
        self.value_balance +=
            Fr::from(note.value().amount) * note.value().asset_id.value_generator();
        self.value_commitments += value_commitment.0;

        let spend_auth_randomizer = Fr::rand(rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let (body, spend_proof) = spend::Body::new(
            value_commitment,
            *spend_key.spend_auth_key(),
            spend_auth_randomizer,
            merkle_path,
            note,
            v_blinding,
            *spend_key.nullifier_key(),
        );

        self.spends.push((rsk, spend_proof, body));

        Ok(self)
    }

    /// Create a new `Output`, implicitly creating a new note for it and encrypting the provided
    /// [`MemoPlaintext`] with a fresh ephemeral secret key.
    ///
    /// To return the generated note, use [`Builder::add_output_producing_note`].
    pub fn add_output<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        dest: &Address,
        value_to_send: Value,
        memo: MemoPlaintext,
        ovk: &OutgoingViewingKey,
    ) -> &mut Self {
        self.add_output_producing_note(rng, dest, value_to_send, memo, ovk);
        self
    }

    /// Generate a new note and add it to the output, returning a clone of the generated note.
    ///
    /// For chaining output, use [`Builder::add_output`].
    pub fn add_output_producing_note<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        dest: &Address,
        value_to_send: Value,
        memo: MemoPlaintext,
        ovk: &OutgoingViewingKey,
    ) -> Note {
        let note = Note::generate(rng, dest, value_to_send);

        let v_blinding = Fr::rand(rng);
        let output = Output::new(rng, note.clone(), memo, dest, ovk, v_blinding);

        // Outputs subtract from the transaction's value balance.
        self.synthetic_blinding_factor -= v_blinding;
        self.value_balance -=
            Fr::from(note.value().amount) * note.value().asset_id.value_generator();

        self.value_commitments += output.body.value_commitment.0;
        self.outputs.push(output);

        note
    }

    /// Adds a `Delegate` description to the transaction.
    pub fn add_delegation(&mut self, delegate: Delegate) -> &mut Self {
        let value_commitment = delegate.value_commitment();
        // The value commitment has 0 blinding factor, so we skip
        // accumulating a blinding term into the synthetic blinding factor.
        self.value_balance += value_commitment.0;
        self.value_commitments += value_commitment.0;

        self.delegations.push(delegate);

        self
    }

    /// Adds an `Undelegate` description to the transaction.
    pub fn add_undelegation(&mut self, undelegate: Undelegate) -> &mut Self {
        let value_commitment = undelegate.value_commitment();
        // The value commitment has 0 blinding factor, so we skip
        // accumulating a blinding term into the synthetic blinding factor.
        self.value_balance += value_commitment.0;
        self.value_commitments += value_commitment.0;

        self.undelegations.push(undelegate);

        self
    }

    pub fn add_validator_definition(&mut self, validator: pbs::ValidatorDefinition) -> &mut Self {
        self.validator_definitions.push(validator);
        self
    }

    /// Set the transaction fee in PEN.
    ///
    /// Note that we're using the lower case `pen` in the code.
    pub fn set_fee(&mut self, fee: u64) -> &mut Self {
        let asset_id = *STAKING_TOKEN_ASSET_ID;
        let fee_value = Value {
            amount: fee,
            asset_id: asset_id.clone(),
        };

        // The fee is effectively an additional output, so we
        // add to the transaction's value balance.
        let value_commitment = fee_value.commit(Fr::zero());
        // The value commitment has 0 blinding factor, so we skip
        // accumulating a blinding term into the synthetic blinding factor.
        self.value_balance -= value_commitment.0;
        self.value_commitments -= value_commitment.0;

        self.fee = Some(Fee(fee));
        self
    }

    /// Set the expiry height.
    pub fn set_expiry_height(&mut self, expiry_height: u64) -> &mut Self {
        self.expiry_height = Some(expiry_height);
        self
    }

    /// Set the chain ID.
    pub fn set_chain_id(&mut self, chain_id: String) -> &mut Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Add the binding signature based on the current sum of synthetic blinding factors.
    #[allow(non_snake_case)]
    pub fn compute_binding_sig<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        auth_hash: &AuthHash,
    ) -> Signature<Binding> {
        let binding_signing_key: SigningKey<Binding> = self.synthetic_blinding_factor.into();

        // Check that the derived verification key corresponds to the signing key to be used.
        let H = value::VALUE_BLINDING_GENERATOR.deref();
        let binding_verification_key_raw = (self.synthetic_blinding_factor * H).compress().0;

        // If value balance is non-zero, the verification key would be value_commitments - value_balance,
        // but value_balance should always be zero.
        let computed_verification_key = self.value_commitments.compress().0;
        assert_eq!(binding_verification_key_raw, computed_verification_key);

        binding_signing_key.sign(rng, auth_hash.as_ref())
    }

    pub fn finalize<R: CryptoRng + RngCore>(
        &mut self,
        mut rng: &mut R,
    ) -> Result<Transaction, Error> {
        if self.chain_id.is_none() {
            return Err(Error::NoChainID);
        }

        if self.fee.is_none() {
            return Err(Error::FeeNotSet);
        }

        if self.value_balance != decaf377::Element::default() {
            return Err(Error::NonZeroValueBalance);
        }

        let mut actions = Vec::<Action>::new();

        // Randomize all actions to minimize info leakage.
        self.spends.shuffle(rng);
        self.outputs.shuffle(rng);
        self.delegations.shuffle(rng);
        self.undelegations.shuffle(rng);
        self.validator_definitions.shuffle(rng);

        // Fill in the spends using blank signatures, so we can build the sighash tx
        for (_, proof, body) in &self.spends {
            actions.push(Action::Spend(Spend {
                body: body.clone(),
                proof: proof.clone(),
                auth_sig: Signature::from([0; 64]),
            }));
        }
        for output in self.outputs.drain(..) {
            actions.push(Action::Output(output));
        }
        for delegation in self.delegations.drain(..) {
            actions.push(Action::Delegate(delegation));
        }
        for undelegation in self.undelegations.drain(..) {
            actions.push(Action::Undelegate(undelegation));
        }
        for vd in &self.validator_definitions {
            // validate the validator signature is signed by the identity key within the validator
            // for a client-side safety check
            // TODO: fix this and other unwraps, by switching from an enumerated error
            // type to anyhow::Error so we don't have to add an extra error variant for
            // everything that can fail.
            let protobuf_serialized: pbs::Validator = vd.validator.clone().unwrap();
            let v_bytes = protobuf_serialized.encode_to_vec();

            // Deserialize the IdentityKey to check the signature.
            let ik: IdentityKey = vd
                .validator
                .clone()
                .unwrap()
                .identity_key
                .unwrap()
                .try_into()
                .unwrap();
            let auth_sig: Signature<SpendAuth> = vd.auth_sig.as_slice().try_into().unwrap();

            ik
                .0
                .verify(&v_bytes, &auth_sig)
                .expect("expected identity key within validator definition to have signed validator definition");
            actions.push(Action::ValidatorDefinition(vd.clone()));
        }

        let mut transaction_body = TransactionBody {
            actions,
            expiry_height: self.expiry_height.unwrap_or(0),
            chain_id: self.chain_id.take().unwrap(),
            fee: self.fee.take().unwrap(),
        };

        // The transaction body is filled except for the signatures,
        // so we can compute the auth hash...
        let auth_hash = transaction_body.auth_hash();

        // and use it to fill in the spendauth sigs...
        for i in 0..self.spends.len() {
            let (rsk, _, _) = self.spends[i];
            if let Action::Spend(Spend {
                ref mut auth_sig, ..
            }) = transaction_body.actions[i]
            {
                *auth_sig = rsk.sign(&mut rng, auth_hash.as_ref());
            } else {
                unreachable!("spends come first in actions list")
            }
        }

        // ... and the binding sig
        let binding_sig = self.compute_binding_sig(rng, &auth_hash);

        // Prevent accidental reuse by erasing the chain ID.
        // It'd be cleaner to take ownership of self and consume it,
        // but that's not possible to chain with &mut self methods, and those
        // are useful when building complex transactions.
        self.chain_id = None;

        Ok(Transaction {
            anchor: self.merkle_root.clone(),
            transaction_body,
            binding_sig,
        })
    }
}
