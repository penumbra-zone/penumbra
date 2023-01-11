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
    // TODO: in some places, we'll want the diversified generator (and
    // transmission key) as a validated
    // curve point, in others we'll want it as the encoding.  which should we
    // pick as the "default" internal representation? for now, use both, and
    // over-constrain, then we can optimize later we could, e.g. have an enum {
    // Unconstrained, Encoding, Element, EncodingAndElement } that does lazy
    // eval of constraints internal mutability on enum, and then have the
    // accessors take &mut self, and then either fetch the already-allocated
    // variable, or allocate it and mutate the internal state to do constraint
    // on demand ?
    diversified_generator: ElementVar,
    // transmission_key: ElementVar,
    transmission_key_s: FqVar,
    // Output proof needs: diversified generator as element and does the elligator
    // map to get an Fq, transmission key as Fq
    // Spend proof needs: diversified generator as element and does the elligator
    // map to get an Fq, transmission key as Fq and element
    clue_key: FqVar,
}

impl AddressVar {
    pub fn diversified_generator(&self) -> ElementVar {
        self.diversified_generator.clone()
    }

    // pub fn transmission_key(&self) -> ElementVar {
    //     self.transmission_key.clone()
    // }

    pub fn transmission_key_s(&self) -> FqVar {
        self.transmission_key_s.clone()
    }

    pub fn clue_key(&self) -> FqVar {
        self.clue_key.clone()
    }
}

impl AllocVar<Address, Fq> for AddressVar {
    fn new_variable<T: std::borrow::Borrow<Address>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let value1 = f()?;
        let address: Address = *value1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let diversified_generator: ElementVar =
                    AllocVar::<Element, Fq>::new_witness(cs.clone(), || {
                        Ok(address.diversified_generator().clone())
                    })?;
                let transmission_key_s =
                    FqVar::new_witness(cs.clone(), || Ok(address.transmission_key_s().clone()))?;
                // dbg!(decaf377::Encoding(address.transmission_key().0).vartime_decompress());
                // let element_transmission_key = decaf377::Encoding(address.transmission_key().0)
                //     .vartime_decompress()
                //     .map_err(|_| SynthesisError::AssignmentMissing)?;
                // let transmission_key: ElementVar =
                //     AllocVar::<Element, Fq>::new_witness(cs.clone(), || {
                //         Ok(element_transmission_key)
                //     })?;
                let clue_key = FqVar::new_witness(cs.clone(), || {
                    Ok(Fq::from_le_bytes_mod_order(&address.clue_key().0[..]))
                })?;

                Ok(Self {
                    cs,
                    diversified_generator,
                    transmission_key_s,
                    // transmission_key,
                    clue_key,
                })
            }
        }
    }
}

pub struct NoteVar {
    cs: ConstraintSystemRef<Fq>,
    value: ValueVar,
    note_blinding: FqVar,
    address: AddressVar,
}

impl NoteVar {
    pub fn amount(&self) -> FqVar {
        self.value.amount()
    }

    pub fn asset_id(&self) -> FqVar {
        self.value.asset_id()
    }

    pub fn note_blinding(&self) -> FqVar {
        self.note_blinding.clone()
    }

    pub fn diversified_generator(&self) -> ElementVar {
        self.address.diversified_generator.clone()
    }

    // pub fn transmission_key(&self) -> ElementVar {
    //     self.address.transmission_key.clone()
    // }

    pub fn transmission_key_s(&self) -> FqVar {
        self.address.transmission_key_s.clone()
    }

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
                let inner = FqVar::new_input(cs.clone(), || Ok(note_commitment.0))?;

                Ok(Self { cs, inner })
            }
            AllocationMode::Witness => unimplemented!(),
        }
    }
}

impl NoteVar {
    pub fn commit(&self) -> Result<NoteCommitmentVar, SynthesisError> {
        let domain_sep = FqVar::new_constant(self.cs.clone(), *NOTECOMMIT_DOMAIN_SEP)?;
        // TODO: should we do this as part of allocating AddressVar? Should only be done
        // if needed as it is expensive. See the comment in the struct def for AddressVar
        let compressed_g_d = self.address.diversified_generator().compress_to_field()?;

        let commitment = poseidon377::r1cs::hash_6(
            self.cs.clone(),
            &domain_sep,
            (
                self.note_blinding.clone(),
                self.value.amount(),
                self.value.asset_id(),
                compressed_g_d,
                self.address.transmission_key_s(),
                self.address.clue_key(),
            ),
        )?;

        Ok(NoteCommitmentVar {
            cs: self.cs.clone(),
            inner: commitment,
        })
    }
}

impl EqGadget<Fq> for NoteCommitmentVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        self.inner.is_eq(&other.inner)
    }
}
