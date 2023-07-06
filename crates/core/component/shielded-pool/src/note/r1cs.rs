use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};
use penumbra_asset::ValueVar;
use penumbra_keys::address::AddressVar;
use penumbra_tct::r1cs::StateCommitmentVar;

use crate::Note;

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
        let note1 = f()?;
        let note: &Note = note1.borrow();
        let note_blinding =
            FqVar::new_variable(cs.clone(), || Ok(note.note_blinding().clone()), mode)?;
        let value = ValueVar::new_variable(cs.clone(), || Ok(note.value().clone()), mode)?;
        let address = AddressVar::new_variable(cs, || Ok(note.address().clone()), mode)?;

        Ok(Self {
            note_blinding,
            value,
            address,
        })
    }
}

impl ToConstraintField<Fq> for Note {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut elements = Vec::new();
        let note_blinding = self.note_blinding();
        elements.extend([note_blinding]);
        elements.extend(self.value().to_field_elements()?);
        elements.extend(self.address().to_field_elements()?);
        Some(elements)
    }
}

// We do not implement `R1CSVar` for `NoteVar` since the associated type
// should be `Note` which we cannot construct from the R1CS variable
// since we do not have the rseed in-circuit.

impl NoteVar {
    pub fn commit(&self) -> Result<StateCommitmentVar, SynthesisError> {
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

        Ok(StateCommitmentVar { inner: commitment })
    }
}
