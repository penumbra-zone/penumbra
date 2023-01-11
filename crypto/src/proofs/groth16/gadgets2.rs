use crate::{
    asset,
    note::{self, NOTECOMMIT_DOMAIN_SEP},
    Address, Amount, Note, Value,
};

use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Element, Fq,
};

pub struct AmountVar {
    cs: ConstraintSystemRef<Fq>,
    amount: FqVar,
}

impl AllocVar<Amount, Fq> for AmountVar {
    fn new_variable<T: std::borrow::Borrow<Amount>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let amount1 = f()?;
        let amount: Amount = *amount1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let inner_amount_var = FqVar::new_witness(cs.clone(), || Ok(Fq::from(amount)))?;
                Ok(Self {
                    cs,
                    amount: inner_amount_var,
                })
            }
        }
    }
}

pub struct AssetIdVar {
    cs: ConstraintSystemRef<Fq>,
    asset_id: FqVar,
}

impl AllocVar<asset::Id, Fq> for AssetIdVar {
    fn new_variable<T: std::borrow::Borrow<asset::Id>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let asset_id1 = f()?;
        let asset_id: asset::Id = *asset_id1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let inner_asset_id_var = FqVar::new_witness(cs.clone(), || Ok(asset_id.0))?;
                Ok(Self {
                    cs,
                    asset_id: inner_asset_id_var,
                })
            }
        }
    }
}

pub struct ValueVar {
    cs: ConstraintSystemRef<Fq>,
    amount: AmountVar,
    asset_id: AssetIdVar,
}

impl AllocVar<Value, Fq> for ValueVar {
    fn new_variable<T: std::borrow::Borrow<Value>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let value1 = f()?;
        let value: Value = *value1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let amount_var = AmountVar::new_witness(cs.clone(), || Ok(value.amount))?;
                let asset_id_var = AssetIdVar::new_witness(cs.clone(), || Ok(value.asset_id))?;
                Ok(Self {
                    cs,
                    amount: amount_var,
                    asset_id: asset_id_var,
                })
            }
        }
    }
}

impl ValueVar {
    pub fn amount(&self) -> FqVar {
        self.amount.amount.clone()
    }

    pub fn asset_id(&self) -> FqVar {
        self.asset_id.asset_id.clone()
    }
}

struct AddressVar {
    cs: ConstraintSystemRef<Fq>,
    // TODO: in some places, we'll want the diversified generator as a validated
    // curve point, in others we'll want it as the encoding.  which should we
    // pick as the "default" internal representation? for now, use both, and
    // over-constrain, then we can optimize later we could, e.g. have an enum {
    // Unconstrained, Encoding, Element, EncodingAndElement } that does lazy
    // eval of constraints internal mutability on enum, and then have the
    // accessors take &mut self, and then either fetch the already-allocated
    // variable, or allocate it and mutate the internal state to do constraint
    // on demand ?
    diversified_generator_s: FqVar,
    transmission_key_s: FqVar,
    // TODO: other fields
}

impl AddressVar {
    pub fn diversified_generator_s(&self) -> FqVar {
        todo!()
    }

    pub fn transmission_key_s(&self) -> FqVar {
        todo!()
    }

    pub fn clue_key_s(&self) -> FqVar {
        todo!()
    }
}

impl AllocVar<Address, Fq> for AddressVar {
    fn new_variable<T: std::borrow::Borrow<Address>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        todo!()
    }
}

struct NoteVar {
    cs: ConstraintSystemRef<Fq>,
    value: ValueVar,
    note_blinding: FqVar,
    address: AddressVar,
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
                let address = AddressVar::new_witness(cs.clone(), || Ok(note.address().clone()))?;

                Ok(Self {
                    cs,
                    note_blinding,
                    value,
                    address,
                })
            }
        }
    }
}

pub struct NoteCommitmentVar {
    cs: ConstraintSystemRef<Fq>,
    inner: FqVar,
}

impl NoteVar {
    pub fn commit(&self) -> Result<NoteCommitmentVar, SynthesisError> {
        let domain_sep = FqVar::new_constant(self.cs.clone(), *NOTECOMMIT_DOMAIN_SEP)?;

        let commitment = poseidon377::r1cs::hash_6(
            self.cs.clone(),
            &domain_sep,
            (
                self.note_blinding.clone(),
                self.value.amount(),
                self.value.asset_id(),
                self.address.diversified_generator_s(),
                self.address.transmission_key_s(),
                self.address.clue_key_s(),
            ),
        )?;

        Ok(NoteCommitmentVar {
            cs: self.cs.clone(),
            inner: commitment,
        })
    }
}

// TODO: impl EqGadget for NoteCommitmentVar
