//! This module defines how to verify TCT auth paths in a rank-1 constraint system.
use ark_ff::ToConstraintField;
use ark_r1cs_std::{prelude::*, uint64::UInt64};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::{r1cs::FqVar, Fq};

use crate::{internal::hash::DOMAIN_SEPARATOR, Position, Proof, StateCommitment};

impl ToConstraintField<Fq> for Position {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        // The variable created in AllocVar<Position, Fq> is a UInt64, which is a
        // Vec of 64 Boolean<Fq> constraints. To construct the corresponding
        // public input, we need to convert the u64 into 64 bits, and then
        // convert each bit into a individual Fq element.
        let mut field_elements = Vec::<Fq>::new();
        let value: u64 = u64::from(*self);
        for i in 0..64 {
            let bit = ((value >> i) & 1) != 0;
            field_elements
                .push(bool::to_field_elements(&bit).expect("can convert bit to field element")[0]);
        }
        Some(field_elements)
    }
}

#[derive(Clone, Debug)]
/// Represents the position of a leaf in the TCT represented in R1CS.
pub struct PositionVar {
    /// The FqVar representing the leaf.
    pub position: FqVar,
    /// Bits
    pub bits: [Boolean<Fq>; 48],
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

        let position = UInt64::new_variable(cs, || Ok(u64::from(inner)), mode)?;
        let bits = position.to_bits_le();
        for bit in &bits[48..] {
            bit.enforce_equal(&Boolean::Constant(false))?;
        }
        let inner = Boolean::<Fq>::le_bits_to_fp_var(&bits[0..48])?;

        Ok(Self {
            bits: bits[0..48]
                .to_vec()
                .try_into()
                .expect("should be able to fit in 48 bits"),
            position: inner,
        })
    }
}

impl ToBitsGadget<Fq> for PositionVar {
    fn to_bits_le(&self) -> Result<Vec<Boolean<Fq>>, SynthesisError> {
        Ok(self.bits.to_vec())
    }
}

impl PositionVar {
    /// Witness the commitment index by taking the last 16 bits of the position.
    pub fn commitment(&self) -> Result<FqVar, SynthesisError> {
        Boolean::<Fq>::le_bits_to_fp_var(&self.bits[0..16])
    }

    /// Witness the block.
    pub fn block(&self) -> Result<FqVar, SynthesisError> {
        Boolean::<Fq>::le_bits_to_fp_var(&self.bits[16..32])
    }

    /// Witness the epoch by taking the first 16 bits of the position.
    pub fn epoch(&self) -> Result<FqVar, SynthesisError> {
        Boolean::<Fq>::le_bits_to_fp_var(&self.bits[32..48])
    }
}

impl R1CSVar<Fq> for PositionVar {
    type Value = Position;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.position.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner_fq = self.position.value()?;
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
        position_bits: &[Boolean<Fq>],
        anchor_var: FqVar,
        commitment_var: FqVar,
    ) -> Result<(), SynthesisError> {
        // We need to compute the root using the provided auth path, position,
        // and leaf.
        let domain_separator = FqVar::new_constant(cs.clone(), *DOMAIN_SEPARATOR)?;
        let leaf_var = poseidon377::r1cs::hash_1(cs.clone(), &domain_separator, commitment_var)?;

        // Height 0 is the leaf.
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
    /// This FqVar has been constructed from two bits of the position.
    inner: FqVar,
}

impl WhichWayVar {
    /// Given a height and an index of a leaf, determine which direction the path down to that leaf
    /// should branch at the node at that height. Allocates a `WhichWayVar`.
    pub fn at(height: u8, position_bits: &[Boolean<Fq>]) -> Result<WhichWayVar, SynthesisError> {
        let shift = 2 * (height - 1);
        let bit_1 = position_bits[shift as usize].clone();
        let bit_2 = position_bits[(shift + 1) as usize].clone();

        // Convert last two bits back to a field element.
        //
        // The below is effectively ensuring that the inner FqVar is constrained to be within
        // the range [0, 3] via the equation `inner = bit_1 + 2 * bit_2`
        // For example, for the maximum values: bit_1 = 1, bit_2 = 1
        // inner = 1 + 2 * 1 = 3
        let inner = FqVar::from(bit_1) + FqVar::constant(Fq::from(2u64)) * FqVar::from(bit_2);

        Ok(WhichWayVar { inner })
    }

    /// Insert the provided node into the quadtree at the provided height.
    pub fn insert(&self, node: FqVar, siblings: [FqVar; 3]) -> Result<[FqVar; 4], SynthesisError> {
        // The node is the leftmost (0th) child.
        let is_leftmost = self.inner.is_eq(&FqVar::zero())?;
        // The node is the left (1st) child.
        let is_left = self.inner.is_eq(&FqVar::one())?;
        // The node is the right (2nd) child.
        let is_right = self.inner.is_eq(&FqVar::constant(Fq::from(2u128)))?;
        // The node is the rightmost (3rd) child.
        let is_rightmost = self.inner.is_eq(&FqVar::constant(Fq::from(3u128)))?;

        // Cases:
        // * `is_leftmost`: the leftmost should be the node
        // * `is_left`: the leftmost should be the first sibling (`siblings[0]`)
        // * `is_right`: the leftmost should be the first sibling (`siblings[0]`)
        // * `is_rightmost`: the leftmost should be the first sibling (`siblings[0]`)
        let leftmost = FqVar::conditionally_select(&is_leftmost, &node, &siblings[0])?;

        // Cases:
        // * `is_leftmost`: the left should be the first sibling (`siblings[0]`)
        // * `is_left`: the left should be the node
        // * `is_right`: the left should be the second sibling (`siblings[1]`)
        // * `is_rightmost`: the left should be the second sibling (`siblings[1]`)
        let is_left_or_leftmost_case = is_leftmost.or(&is_left)?;
        let left_first_two_cases = FqVar::conditionally_select(&is_left, &node, &siblings[0])?;
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
        let is_right_or_rightmost_case = is_right.or(&is_rightmost)?;
        let right_last_two_cases = FqVar::conditionally_select(&is_right, &node, &siblings[2])?;
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
        let rightmost = FqVar::conditionally_select(&is_rightmost, &node, &siblings[2])?;

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
