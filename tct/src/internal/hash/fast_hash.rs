//! This module, when enabled with the `fast_hash` feature flag, replaces the Poseidon hash
//! implementation with `blake2b_simd`, which is much faster. **This is useless for production use
//! of this crate; it is only useful to accelerate testing, when we don't care about producing
//! zero-knowledge proofs.**

use ark_ff::fields::PrimeField;
use decaf377::FieldExt;
use poseidon377::Fq;
