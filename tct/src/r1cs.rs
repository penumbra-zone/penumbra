//! This module defines how to verify TCT auth paths in a rank-1 constraint system.
use ark_ff::Zero;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::{r1cs::FqVar, FieldExt, Fq};

use crate::{internal::hash::DOMAIN_SEPARATOR, prelude::WhichWay, Position, Proof};

/// Represents the position of a leaf in the TCT represented in R1CS.
pub struct PositionVar {
    /// The FqVar representing the leaf.
    pub inner: FqVar,
}

impl AllocVar<Position, Fq> for PositionVar {
    fn new_variable<T: std::borrow::Borrow<Position>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: Position = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => Ok(Self {
                inner: FqVar::new_witness(cs, || Ok(Fq::from(u64::from(inner))))?,
            }),
        }
    }
}

impl R1CSVar<Fq> for PositionVar {
    type Value = Position;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner_fq = self.inner.value()?;
        let inner_bytes = &inner_fq.to_bytes()[0..16];
        let position_bytes: [u8; 8] = inner_bytes
            .try_into()
            .expect("should be able to fit in 16 bytes");
        Ok(Position::from(u64::from_le_bytes(position_bytes)))
    }
}

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
        commitment_var: FqVar,
    ) -> Result<(), SynthesisError> {
        // We need to compute the root using the provided auth path, position,
        // and leaf.
        let domain_separator = FqVar::new_constant(cs.clone(), *DOMAIN_SEPARATOR)?;
        let leaf_var = poseidon377::r1cs::hash_1(cs.clone(), &domain_separator, commitment_var)?;

        // Height 0 is the commitment.
        let mut previous_level = leaf_var;

        // Start hashing from height 1, first hashing the leaf and its three siblings together,
        // then the next level and so on, until we reach the root of the quadtree.
        for height_value in 1..=24 {
            // Check which way to go.
            let index_fq = position_var.value().unwrap_or_else(|_| Fq::zero());
            let index_value = u64::from_le_bytes(
                index_fq.to_bytes()[0..8]
                    .try_into()
                    .expect("index value should always fit in a u64"),
            );
            let which_way = WhichWay::at(height_value, index_value).0;
            let which_way_var = WhichWayVar::new(cs.clone(), which_way)?;

            let height_var = FqVar::new_constant(cs.clone(), Fq::from(height_value as u64))?;

            let siblings = &self.inner[(24 - height_value) as usize];
            let [leftmost, left, right, rightmost] =
                which_way_var.insert(previous_level.clone(), siblings.clone())?;

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

/// Represents the different paths a quadtree node can take.
///
/// A bundle of boolean R1CS constraints representing the path.
pub struct WhichWayVar {
    /// The node is the leftmost (0th) child.
    pub is_leftmost: Boolean<Fq>,
    /// The node is the left (1st) child.
    pub is_left: Boolean<Fq>,
    /// The node is the right (2nd) child.
    pub is_right: Boolean<Fq>,
    /// The node is the rightmost (3rd) child.
    pub is_rightmost: Boolean<Fq>,
}

impl WhichWayVar {
    /// Create an R1CS `WhichWayVar` from a `WhichWay` enum.
    pub fn new(
        cs: ConstraintSystemRef<Fq>,
        which_way: WhichWay,
    ) -> Result<WhichWayVar, SynthesisError> {
        // TODO: impl AllocVar
        match which_way {
            WhichWay::Leftmost => Ok(WhichWayVar {
                is_leftmost: Boolean::new_witness(cs.clone(), || Ok(true))?,
                is_left: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_right: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_rightmost: Boolean::new_witness(cs, || Ok(false))?,
            }),
            WhichWay::Left => Ok(WhichWayVar {
                is_leftmost: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_left: Boolean::new_witness(cs.clone(), || Ok(true))?,
                is_right: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_rightmost: Boolean::new_witness(cs, || Ok(false))?,
            }),
            WhichWay::Right => Ok(WhichWayVar {
                is_leftmost: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_left: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_right: Boolean::new_witness(cs.clone(), || Ok(true))?,
                is_rightmost: Boolean::new_witness(cs, || Ok(false))?,
            }),
            WhichWay::Rightmost => Ok(WhichWayVar {
                is_leftmost: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_left: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_right: Boolean::new_witness(cs.clone(), || Ok(false))?,
                is_rightmost: Boolean::new_witness(cs, || Ok(true))?,
            }),
        }
    }

    /// Insert the provided node into the quadtree at the provided height.
    pub fn insert(&self, node: FqVar, siblings: [FqVar; 3]) -> Result<[FqVar; 4], SynthesisError> {
        // Cases:
        // * `is_leftmost`: the leftmost should be the node
        // * `is_left`: the leftmost should be the first sibling (`siblings[0]`)
        // * `is_right`: the leftmost should be the first sibling (`siblings[0]`)
        // * `is_rightmost`: the leftmost should be the first sibling (`siblings[0]`)
        let leftmost = FqVar::conditionally_select(&self.is_leftmost, &node, &siblings[0])?;

        // Cases:
        // * `is_leftmost`: the left should be the first sibling (`siblings[0]`)
        // * `is_left`: the left should be the node
        // * `is_right`: the left should be the second sibling (`siblings[1]`)
        // * `is_rightmost`: the left should be the second sibling (`siblings[1]`)
        let is_left_or_leftmost_case = self.is_leftmost.or(&self.is_left)?;
        let left_first_two_cases = FqVar::conditionally_select(&self.is_left, &node, &siblings[0])?;
        let left = FqVar::conditionally_select(
            &is_left_or_leftmost_case,
            &left_first_two_cases,
            &siblings[1],
        )?;

        // Cases:
        // * `is_leftmost`: the right should be the second sibling (`siblings[1]`)
        // * `is_left`: the right should be the second sibling (`siblings[1]`)
        // * `is_right`: the right should be the node
        // * `is_rightmost`: the right should be the last sibling (`siblings[2]`)
        let is_right_or_rightmost_case = self.is_right.or(&self.is_rightmost)?;
        let right_last_two_cases =
            FqVar::conditionally_select(&self.is_right, &node, &siblings[2])?;
        let right = FqVar::conditionally_select(
            &is_right_or_rightmost_case,
            &right_last_two_cases,
            &siblings[1],
        )?;

        // Cases:
        // * `is_leftmost`: the rightmost should be the last sibling (`siblings[2]`)
        // * `is_left`: the rightmost should be the last sibling (`siblings[2]`)
        // * `is_right`: the rightmost should be the last sibling (`siblings[2]`)
        // * `is_rightmost`: the rightmost should be the node
        let rightmost = FqVar::conditionally_select(&self.is_rightmost, &node, &siblings[2])?;

        Ok([leftmost, left, right, rightmost])
    }
}
