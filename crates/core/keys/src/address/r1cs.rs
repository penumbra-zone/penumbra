use crate::Address;
use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Element, Fq,
};

#[derive(Clone)]
pub struct AddressVar {
    pub diversified_generator: ElementVar,
    pub transmission_key: ElementVar,
    pub clue_key: FqVar,
}

impl AddressVar {
    pub fn diversified_generator(&self) -> ElementVar {
        self.diversified_generator.clone()
    }

    pub fn transmission_key(&self) -> ElementVar {
        self.transmission_key.clone()
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
        let address: Address = *f()?.borrow();

        let diversified_generator: ElementVar = AllocVar::<Element, Fq>::new_variable(
            cs.clone(),
            || Ok(address.diversified_generator()),
            mode,
        )?;
        let element_transmission_key = decaf377::Encoding(address.transmission_key().0)
            .vartime_decompress()
            .map_err(|_| SynthesisError::AssignmentMissing)?;
        let transmission_key: ElementVar = AllocVar::<Element, Fq>::new_variable(
            cs.clone(),
            || Ok(element_transmission_key),
            mode,
        )?;
        let clue_key = FqVar::new_variable(
            cs,
            || Ok(Fq::from_le_bytes_mod_order(&address.clue_key().0[..])),
            mode,
        )?;

        Ok(Self {
            diversified_generator,
            transmission_key,
            clue_key,
        })
    }
}

// We do not implement the R1CSVar trait for AddressVar since we do not have the
// diversifier in-circuit in order to construct an Address.

impl ToConstraintField<Fq> for Address {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut elements = Vec::new();
        elements.extend(self.diversified_generator().to_field_elements()?);
        let transmission_key_fq = decaf377::Encoding(self.transmission_key().0)
            .vartime_decompress()
            .expect("transmission key is valid decaf377 Element");
        elements.extend([transmission_key_fq.vartime_compress_to_field()]);
        elements.extend(Fq::from_bytes_checked(&self.clue_key().0));
        Some(elements)
    }
}
