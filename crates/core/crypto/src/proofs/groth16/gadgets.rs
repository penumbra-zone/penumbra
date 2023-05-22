use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{r1cs::ElementVar, Fq};

/// Check the element is not identity.
pub(crate) fn element_not_identity(
    cs: ConstraintSystemRef<Fq>,
    enforce: &Boolean<Fq>,
    // Witness
    element: ElementVar,
) -> Result<(), SynthesisError> {
    let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
    identity.conditional_enforce_not_equal(&element, enforce)?;
    Ok(())
}
