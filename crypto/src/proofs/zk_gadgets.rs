use anyhow::Result;

use ark_ff::Field;
use ark_relations::r1cs::ConstraintSystemRef;

pub trait Decaf377Gadget {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()>;
}

impl<F: Field> Decaf377Gadget for ConstraintSystemRef<F> {
    fn new_decaf377_input_variable(self, element: decaf377::Element) -> Result<()> {
        todo!()
    }
}

pub fn diversified_basepoint_not_identity<F: Field>(mut cs: ConstraintSystemRef<F>) {
    todo!()
}
