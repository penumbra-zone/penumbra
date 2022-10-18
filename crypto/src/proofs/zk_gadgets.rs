use anyhow::Result;

use ark_ff::Field;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::{fields::fp::FpVar, prelude::AllocVar};
use ark_relations::{ns, r1cs::ConstraintSystemRef};
use decaf377::Fq;

pub trait Decaf377Gadget {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()>;
}

impl<F: Field> Decaf377Gadget for ConstraintSystemRef<F> {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()> {
        todo!()
    }
}

pub fn diversified_basepoint_not_identity<F: Field>(cs: ConstraintSystemRef<F>) -> Result<()> {
    // TODO: use `new_decaf377_input_variable` instead of bare Fq here

    // 1. Add identity element
    let identity_fq: Fq = decaf377::Element::default().vartime_compress_to_field();
    let identity = FpVar::<Fq>::new_constant(ns!(cs, "decaf_identity"), identity_fq)?;

    // 2. Add diversified base
    // TK

    // Add not equality constraint using EqGadget.
    //identity.enforce_not_equal();

    Ok(())
}
