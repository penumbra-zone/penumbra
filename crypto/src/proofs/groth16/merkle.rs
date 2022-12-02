use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{r1cs::FqVar, Fq};

use penumbra_tct as tct;

/// This represents the TCT's auth path in R1CS.
pub struct MerkleAuthPathVar {
    inner: [[FqVar; 3]; 24],
}

impl MerkleAuthPathVar {
    /// Witness a TCT auth path.
    pub fn new(cs: ConstraintSystemRef<Fq>, tct_proof: tct::Proof) -> Result<Self, SynthesisError> {
        let mut auth_path = Vec::<[FqVar; 3]>::new();
        for depth in tct_proof.auth_path() {
            let mut nodes = [FqVar::zero(), FqVar::zero(), FqVar::zero()];
            for (i, node) in depth.iter().enumerate() {
                nodes[i] = FqVar::new_witness(cs.clone(), || Ok(Fq::from(node.clone())))?;
            }
            auth_path.push(nodes);
        }
        Ok(Self {
            inner: auth_path
                .try_into()
                .expect("TCT auth path should have depth 24"),
        })
    }

    /// Certify an auth path given a provided anchor, position, and leaf.
    pub fn verify(
        &self,
        cs: ConstraintSystemRef<Fq>,
        enforce: &Boolean<Fq>,
        position_var: FqVar,
        anchor_var: FqVar,
        leaf_var: FqVar,
    ) -> Result<(), SynthesisError> {
        // We need to compute the root using the provided auth path, position,
        // and leaf.
        let domain_separator = FqVar::new_constant(cs.clone(), *tct::DOMAIN_SEPARATOR)?;
        let leaf_var = poseidon377::r1cs::hash_1(cs.clone(), &domain_separator, leaf_var)?;
        let computed_root = anchor_var.clone();
        // TODO: Compute root

        anchor_var.conditional_enforce_equal(&computed_root, &enforce)?;

        Ok(())
    }
}
