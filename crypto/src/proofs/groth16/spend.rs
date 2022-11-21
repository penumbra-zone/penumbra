use std::str::FromStr;

use ark_r1cs_std::uint8::UInt8;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Bls12_377, Fq, Fr,
};
use decaf377::{Element, FieldExt};

use ark_ff::{PrimeField, ToConstraintField};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use rand_core::OsRng;

use crate::proofs::groth16_gadgets as gadgets;
use crate::{
    balance,
    keys::{NullifierKey, SeedPhrase, SpendKey},
    Note, Nullifier, Value,
};

/// Groth16 proof for spending existing notes.
#[derive(Clone, Debug)]
pub struct SpendCircuit {
    // Witnesses
    /// Inclusion proof for the note commitment.
    note_commitment_proof: tct::Proof,
    /// The note being spent.
    note: Note,
    /// The blinding factor used for generating the value commitment.
    v_blinding: Fr,
    /// The randomizer used for generating the randomized spend auth key.
    spend_auth_randomizer: Fr,
    /// The spend authorization key.
    ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    nk: NullifierKey,

    // Public inputs
    /// the merkle root of the note commitment tree.
    pub anchor: tct::Root,
    /// value commitment of the note to be spent.
    pub balance_commitment: balance::Commitment,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
}

impl ConstraintSynthesizer<Fq> for SpendCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        // TODO: tct::Proof unpack into position, note commitment, and the merkle path
        let note_commitment_var =
            FqVar::new_witness(cs.clone(), || Ok(self.note_commitment_proof.commitment().0))?;
        let position_fq: Fq = Fq::from(u64::from(self.note_commitment_proof.position()));
        let nct_position_var = FqVar::new_witness(cs.clone(), || Ok(position_fq))?;

        let note_blinding_var =
            FqVar::new_witness(cs.clone(), || Ok(self.note.note_blinding().clone()))?;
        let value_amount_var =
            FqVar::new_witness(cs.clone(), || Ok(Fq::from(self.note.value().amount)))?;
        let value_asset_id_var =
            FqVar::new_witness(cs.clone(), || Ok(self.note.value().asset_id.0))?;
        let diversified_generator_var: ElementVar =
            AllocVar::<Element, Fq>::new_witness(cs.clone(), || {
                Ok(self.note.diversified_generator().clone())
            })?;
        let transmission_key_s_var =
            FqVar::new_witness(cs.clone(), || Ok(self.note.transmission_key_s().clone()))?;
        let element_transmission_key = decaf377::Encoding(self.note.transmission_key().0)
            .vartime_decompress()
            .map_err(|err| SynthesisError::AssignmentMissing)?;
        let transmission_key_var: ElementVar =
            AllocVar::<Element, Fq>::new_witness(cs.clone(), || Ok(element_transmission_key))?;
        let clue_key_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from_le_bytes_mod_order(&self.note.clue_key().0[..]))
        })?;
        let v_blinding_arr: [u8; 32] = self.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;
        let value_amount_arr = self.note.value().amount.to_le_bytes();
        let value_vars = UInt8::new_witness_vec(cs.clone(), &value_amount_arr)?;
        // TODO: spend_auth_randomizer
        let ak_bytes = Fq::from_bytes(*self.ak.as_ref())
            .expect("verification key is valid, so its byte encoding is a decaf377 s value");
        let ak_var = FqVar::new_witness(cs.clone(), || Ok(ak_bytes))?;
        let nk_var = FqVar::new_witness(cs.clone(), || Ok(self.nk.0))?;

        // Public inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
        let balance_commitment_var =
            ElementVar::new_input(cs.clone(), || Ok(self.balance_commitment.0))?;
        let nullifier_var = FqVar::new_input(cs.clone(), || Ok(self.nullifier.0))?;
        // TODO: rk

        // TODO: Short circuit to true if value released is 0. That means this is a _dummy_ spend.

        gadgets::note_commitment_integrity(
            cs.clone(),
            note_blinding_var,
            value_amount_var,
            value_asset_id_var.clone(),
            diversified_generator_var.clone(),
            transmission_key_s_var,
            clue_key_var,
            note_commitment_var.clone(),
        )?;
        // TODO: Merkle path integrity.
        // TODO: rk integrity
        gadgets::diversified_address_integrity(
            cs.clone(),
            ak_var,
            nk_var.clone(),
            transmission_key_var,
            diversified_generator_var.clone(),
        )?;
        gadgets::diversified_basepoint_not_identity(cs.clone(), diversified_generator_var)?;
        gadgets::value_commitment_integrity(
            cs.clone(),
            value_vars,
            value_asset_id_var,
            v_blinding_vars,
            balance_commitment_var,
        )?;
        gadgets::nullifier_integrity(
            cs,
            note_commitment_var,
            nk_var,
            nct_position_var,
            nullifier_var,
        )?;

        Ok(())
    }
}

impl SpendCircuit {
    pub fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let spend_auth_randomizer = Fr::from(1);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Fq::from(1),
        )
        .expect("can make a note");
        let v_blinding = Fr::from(1);
        let rk = rsk.into();
        let nullifier = Nullifier(Fq::from(1));
        let mut nct = tct::Tree::new();
        let note_commitment = note.commit();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();

        let circuit = SpendCircuit {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment: balance::Commitment(decaf377::basepoint()),
            nullifier,
            rk,
        };
        let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut OsRng)
            .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

pub struct SpendProof(Proof<Bls12_377>);

impl SpendProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        note_commitment_proof: tct::Proof,
        note: Note,
        v_blinding: Fr,
        spend_auth_randomizer: Fr,
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
    ) -> anyhow::Result<Self> {
        let circuit = SpendCircuit {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nullifier,
            rk,
        };
        let proof = Groth16::prove(pk, circuit, rng).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    pub fn verify(
        &self,
        vk: &VerifyingKey<Bls12_377>,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
    ) -> anyhow::Result<bool> {
        let processed_pvk = Groth16::process_vk(&vk).map_err(|err| anyhow::anyhow!(err))?;
        //let element_pk = decaf377::Encoding(epk.0).vartime_decompress().unwrap();
        let mut public_inputs = Vec::new();
        public_inputs.extend(Fq::from(anchor.0).to_field_elements().unwrap());
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(nullifier.0.to_field_elements().unwrap());
        //public_inputs.extend(rk.0.to_field_elements().unwrap());

        let proof_result =
            Groth16::verify_with_processed_vk(&processed_pvk, public_inputs.as_slice(), &self.0)
                .map_err(|err| anyhow::anyhow!(err))?;
        Ok(proof_result)
    }
}
