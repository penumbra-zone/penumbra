use poseidon377::hash_3;

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::r1cs::FqVar;
use penumbra_tct as tct;

use crate::{
    note::{Commitment, NoteCommitmentVar},
    nullifier::{Nullifier, NullifierVar, NULLIFIER_DOMAIN_SEP},
    Fq,
};

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a positioned piece of state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NullifierKey(pub Fq);

impl NullifierKey {
    pub fn derive_nullifier(
        &self,
        pos: penumbra_tct::Position,
        state_commitment: &Commitment,
    ) -> Nullifier {
        Nullifier(hash_3(
            &NULLIFIER_DOMAIN_SEP,
            (self.0, state_commitment.0, (u64::from(pos)).into()),
        ))
    }
}

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

impl NullifierKeyVar {
    pub fn derive_nullifier(
        &self,
        position: &tct::r1cs::PositionVar,
        note_commitment: &NoteCommitmentVar,
    ) -> Result<NullifierVar, SynthesisError> {
        let cs = note_commitment.inner.cs();
        let domain_sep = FqVar::new_constant(cs.clone(), *NULLIFIER_DOMAIN_SEP)?;
        let nullifier = poseidon377::r1cs::hash_3(
            cs,
            &domain_sep,
            (
                self.inner.clone(),
                note_commitment.inner.clone(),
                position.inner.clone(),
            ),
        )?;

        Ok(NullifierVar { inner: nullifier })
    }
}
