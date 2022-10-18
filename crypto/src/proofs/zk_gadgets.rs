use anyhow::Result;

use ark_ff::Field;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::{fields::fp::FpVar, prelude::AllocVar};
use ark_relations::{
    ns,
    r1cs::{ConstraintSystemRef, Variable},
};
use decaf377::Fq;

pub trait Decaf377Gadget {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()>;
}

impl<F: Field> Decaf377Gadget for ConstraintSystemRef<F> {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()> {
        todo!()
    }
}
