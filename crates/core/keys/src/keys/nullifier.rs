use decaf377::Fq;

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::r1cs::FqVar;

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a positioned piece of state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NullifierKey(pub Fq);

/// Represents the `NullifierKey` as a variable in an R1CS constraint system.
pub struct NullifierKeyVar {
    pub inner: FqVar,
}

impl AllocVar<NullifierKey, Fq> for NullifierKeyVar {
    fn new_variable<T: std::borrow::Borrow<NullifierKey>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: NullifierKey = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => Ok(Self {
                inner: FqVar::new_witness(cs, || Ok(inner.0))?,
            }),
        }
    }
}

impl R1CSVar<Fq> for NullifierKeyVar {
    type Value = NullifierKey;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner_fq = self.inner.value()?;
        Ok(NullifierKey(inner_fq))
    }
}
