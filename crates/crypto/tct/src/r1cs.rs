//! This module defines how to verify TCT auth paths in a rank-1 constraint system.
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::uint64::UInt64;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::{r1cs::FqVar, FieldExt, Fq};

use crate::{internal::hash::DOMAIN_SEPARATOR, Position, Proof, StateCommitment};

#[derive(Clone, Debug)]
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
        Ok(Self {
            inner: FqVar::new_variable(cs, || Ok(Fq::from(u64::from(inner))), mode)?,
        })
    }
}

#[derive(Clone, Debug)]
/// Represents the position of a leaf in the TCT represented in R1CS.
pub struct PositionBitsVar {
    /// Inner variable consisting of boolean constraints.
    pub inner: Vec<Boolean<Fq>>,
}

impl AllocVar<Position, Fq> for PositionBitsVar {
    fn new_variable<T: std::borrow::Borrow<Position>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: Position = *f()?.borrow();
        let var = UInt64::new_variable(cs, || Ok(u64::from(inner)), mode)?;
        Ok(Self {
            inner: var.to_bits_le(),
        })
    }
}

impl ToBitsGadget<Fq> for PositionBitsVar {
    fn to_bits_le(&self) -> Result<Vec<Boolean<Fq>>, SynthesisError> {
        Ok(self.inner.clone())
    }
}

impl PositionVar {
    /// Get bits of the position.
    pub fn to_position_bits_var(&self) -> Result<PositionBitsVar, SynthesisError> {
        Ok(PositionBitsVar {
            inner: self.inner.to_bits_le()?,
        })
    }
}

impl ToBitsGadget<Fq> for PositionVar {
    fn to_bits_le(&self) -> Result<Vec<Boolean<Fq>>, SynthesisError> {
        self.inner.to_bits_le()
    }
}

impl PositionBitsVar {
    /// Witness the commitment index by taking the last 16 bits of the position.
    pub fn commitment(&self) -> Result<FqVar, SynthesisError> {
        Ok(Boolean::<Fq>::le_bits_to_fp_var(&self.inner[48..64])?)
    }

    /// Witness the block.
    pub fn block(&self) -> Result<FqVar, SynthesisError> {
        Ok(Boolean::<Fq>::le_bits_to_fp_var(&self.inner[16..32])?)
    }

    /// Witness the epoch by taking the first 16 bits of the position.
    pub fn epoch(&self) -> Result<FqVar, SynthesisError> {
        Ok(Boolean::<Fq>::le_bits_to_fp_var(&self.inner[0..16])?)
    }
}

impl R1CSVar<Fq> for PositionVar {
    type Value = Position;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner_fq = self.inner.value()?;
        let inner_bytes = &inner_fq.to_bytes()[0..8];
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

impl AllocVar<Proof, Fq> for MerkleAuthPathVar {
    fn new_variable<T: std::borrow::Borrow<Proof>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner1 = f()?;
        let inner: &Proof = inner1.borrow();
        // This adds one FqVar per sibling and keeps them grouped together by height.
        let mut auth_path = Vec::<[FqVar; 3]>::new();
        for depth in inner.auth_path() {
            let mut nodes = [FqVar::zero(), FqVar::zero(), FqVar::zero()];
            for (i, node) in depth.iter().enumerate() {
                nodes[i] = FqVar::new_variable(cs.clone(), || Ok(Fq::from(*node)), mode)?;
            }
            auth_path.push(nodes);
        }

        Ok(Self {
            inner: auth_path
                .try_into()
                .expect("TCT auth path should have depth 24"),
        })
    }
}

impl MerkleAuthPathVar {
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
        position_bits: &Vec<Boolean<Fq>>,
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
            let height_var = FqVar::new_constant(cs.clone(), Fq::from(height_value as u64))?;
            let which_way_var = WhichWayVar::at(height_value, position_bits)?;
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
    /// Given a height and an index of a leaf, determine which direction the path down to that leaf
    /// should branch at the node at that height. Allocates a `WhichWayVar`.
    pub fn at(height: u8, position_bits: &Vec<Boolean<Fq>>) -> Result<WhichWayVar, SynthesisError> {
        let shift = 2 * (height - 1);
        let index_1 = shift;
        let index_2 = shift + 1;
        let bit_1 = position_bits[index_1 as usize].clone();
        let bit_2 = position_bits[index_2 as usize].clone();

        // Convert last two bits back to a field element.
        let num_last_two_bits =
            FqVar::from(bit_1) + FqVar::constant(Fq::from(2)) * FqVar::from(bit_2);

        let is_leftmost = num_last_two_bits.is_eq(&FqVar::zero())?;
        let is_left = num_last_two_bits.is_eq(&FqVar::one())?;
        let is_right = num_last_two_bits.is_eq(&FqVar::constant(Fq::from(2u128)))?;
        let is_rightmost = num_last_two_bits.is_eq(&FqVar::constant(Fq::from(3u128)))?;

        Ok(WhichWayVar {
            is_leftmost,
            is_left,
            is_right,
            is_rightmost,
        })
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

/// Represents a state commitment in R1CS.
pub struct StateCommitmentVar {
    /// The `FqVar` representing the state commitment.
    pub inner: FqVar,
}

impl StateCommitmentVar {
    /// Access the inner `FqVar`.
    pub fn inner(&self) -> FqVar {
        self.inner.clone()
    }
}

impl AllocVar<StateCommitment, Fq> for StateCommitmentVar {
    fn new_variable<T: std::borrow::Borrow<StateCommitment>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => {
                let note_commitment1 = f()?;
                let note_commitment: StateCommitment = *note_commitment1.borrow();
                let inner = FqVar::new_input(cs, || Ok(note_commitment.0))?;

                Ok(Self { inner })
            }
            AllocationMode::Witness => {
                let note_commitment1 = f()?;
                let note_commitment: StateCommitment = *note_commitment1.borrow();
                let inner = FqVar::new_witness(cs, || Ok(note_commitment.0))?;

                Ok(Self { inner })
            }
        }
    }
}

impl R1CSVar<Fq> for StateCommitmentVar {
    type Value = StateCommitment;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner = self.inner.value()?;
        Ok(StateCommitment(inner))
    }
}

impl EqGadget<Fq> for StateCommitmentVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        self.inner.is_eq(&other.inner)
    }
}
