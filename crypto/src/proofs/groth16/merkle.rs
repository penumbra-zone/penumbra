use ark_r1cs_std::{prelude::*, ToBitsGadget};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{r1cs::FqVar, Fq};

use penumbra_tct as tct;

/// This represents the TCT's auth path in R1CS.
pub struct MerkleAuthPathVar {
    inner: [[FqVar; 3]; 24],
}

impl MerkleAuthPathVar {
    pub fn new(cs: ConstraintSystemRef<Fq>, tct_proof: tct::Proof) -> Result<Self, SynthesisError> {
        todo!()
    }
}
