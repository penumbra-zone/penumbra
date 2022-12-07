//! This module defines how to verify TCT auth paths in a rank-1 constraint system.
use ark_ff::Zero;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::FieldExt;
use decaf377::{r1cs::FqVar, Fq};

use crate::{
    internal::{hash::DOMAIN_SEPARATOR, path::WhichWay},
    Proof,
};

/// This represents the TCT's auth path in R1CS.
pub struct MerkleAuthPathVar {
    inner: [[FqVar; 3]; 24],
}

impl MerkleAuthPathVar {
    /// Witness a TCT auth path.
    ///
    /// This adds one FqVar per sibling and keeps them grouped together by height.
    pub fn new(cs: ConstraintSystemRef<Fq>, tct_proof: Proof) -> Result<Self, SynthesisError> {
        let mut auth_path = Vec::<[FqVar; 3]>::new();
        for depth in tct_proof.auth_path() {
            let mut nodes = [FqVar::zero(), FqVar::zero(), FqVar::zero()];
            for (i, node) in depth.iter().enumerate() {
                nodes[i] = FqVar::new_witness(cs.clone(), || Ok(Fq::from(*node)))?;
            }
            auth_path.push(nodes);
        }

        Ok(Self {
            inner: auth_path
                .try_into()
                .expect("TCT auth path should have depth 24"),
        })
    }

    /// Hash a node given the children at the given height.
    pub fn hash_node(
        cs: ConstraintSystemRef<Fq>,
        height_var: FqVar,
        a: FqVar,
        b: FqVar,
        c: FqVar,
        d: FqVar,
    ) -> Result<FqVar, SynthesisError> {
        let domain_separator = FqVar::new_constant(cs.clone(), *DOMAIN_SEPARATOR)?;
        poseidon377::r1cs::hash_4(cs, &(domain_separator + height_var), (a, b, c, d))
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
        let domain_separator = FqVar::new_constant(cs.clone(), *DOMAIN_SEPARATOR)?;
        let leaf_var = poseidon377::r1cs::hash_1(cs.clone(), &domain_separator, leaf_var)?;

        let index_fq = position_var.value().unwrap_or_else(|_| Fq::zero());
        let index_value = u64::from_le_bytes(
            index_fq.to_bytes()[0..8]
                .try_into()
                .expect("index value should always fit in a u64"),
        );

        // Height 0 is the commitment.
        let mut previous_level = leaf_var;

        // Start hashing from height 1, first hashing the leaf and its three siblings together,
        // then the next level and so on, until we reach the root of the quadtree.
        for height_value in 1..=24 {
            dbg!(height_value);
            let which_way = WhichWay::at(height_value, index_value).0;
            let siblings = &self.inner[(24 - height_value) as usize];
            let [leftmost, left, right, rightmost] =
                which_way.insert(previous_level.clone(), siblings.clone());

            let height_var = FqVar::new_constant(cs.clone(), Fq::from(height_value))?;
            let parent = MerkleAuthPathVar::hash_node(
                cs.clone(),
                height_var,
                leftmost,
                left,
                right,
                rightmost,
            )?;

            previous_level = parent;
        }

        anchor_var.conditional_enforce_equal(&previous_level, enforce)?;

        Ok(())
    }
}
