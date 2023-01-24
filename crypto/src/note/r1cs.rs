use crate::{address::AddressVar, note, value::ValueVar, Note};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};

use super::NOTECOMMIT_DOMAIN_SEP;

pub struct NoteVar {
    pub value: ValueVar,
    pub note_blinding: FqVar,
    pub address: AddressVar,
}

impl NoteVar {
    pub fn amount(&self) -> FqVar {
        self.value.amount()
    }

    pub fn value(&self) -> ValueVar {
        self.value.clone()
    }

    #[allow(dead_code)]
    pub fn asset_id(&self) -> FqVar {
        self.value.asset_id()
    }

    #[allow(dead_code)]
    pub fn note_blinding(&self) -> FqVar {
        self.note_blinding.clone()
    }

    pub fn diversified_generator(&self) -> ElementVar {
        self.address.diversified_generator.clone()
    }

    pub fn transmission_key(&self) -> ElementVar {
        self.address.transmission_key.clone()
    }

    #[allow(dead_code)]
    pub fn clue_key(&self) -> FqVar {
        self.address.clue_key.clone()
    }
}

impl AllocVar<Note, Fq> for NoteVar {
    fn new_variable<T: std::borrow::Borrow<Note>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        // TODO: figure out how to use namespaces
        let ns = cs.into();
        let cs = ns.cs();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let note1 = f()?;
                let note = note1.borrow();

                let note_blinding =
                    FqVar::new_witness(cs.clone(), || Ok(note.note_blinding().clone()))?;
                let value = ValueVar::new_witness(cs.clone(), || Ok(note.value().clone()))?;
                let address = AddressVar::new_witness(cs, || Ok(note.address().clone()))?;

                Ok(Self {
                    note_blinding,
                    value,
                    address,
                })
            }
        }
    }
}

pub struct NoteCommitmentVar {
    pub inner: FqVar,
}

impl NoteCommitmentVar {
    pub fn inner(&self) -> FqVar {
        self.inner.clone()
    }
}

impl AllocVar<note::Commitment, Fq> for NoteCommitmentVar {
    fn new_variable<T: std::borrow::Borrow<note::Commitment>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => {
                let note_commitment1 = f()?;
                let note_commitment: note::Commitment = *note_commitment1.borrow();
                let inner = FqVar::new_input(cs, || Ok(note_commitment.0))?;

                Ok(Self { inner })
            }
            AllocationMode::Witness => {
                let note_commitment1 = f()?;
                let note_commitment: note::Commitment = *note_commitment1.borrow();
                let inner = FqVar::new_witness(cs, || Ok(note_commitment.0))?;

                Ok(Self { inner })
            }
        }
    }
}

impl NoteVar {
    pub fn commit(&self) -> Result<NoteCommitmentVar, SynthesisError> {
        let cs = self.amount().cs();
        let domain_sep = FqVar::new_constant(cs.clone(), *NOTECOMMIT_DOMAIN_SEP)?;
        let compressed_g_d = self.address.diversified_generator().compress_to_field()?;

        let commitment = poseidon377::r1cs::hash_6(
            cs,
            &domain_sep,
            (
                self.note_blinding.clone(),
                self.value.amount(),
                self.value.asset_id(),
                compressed_g_d,
                self.address.transmission_key().compress_to_field()?,
                self.address.clue_key(),
            ),
        )?;

        Ok(NoteCommitmentVar { inner: commitment })
    }
}

impl EqGadget<Fq> for NoteCommitmentVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        self.inner.is_eq(&other.inner)
    }
}
