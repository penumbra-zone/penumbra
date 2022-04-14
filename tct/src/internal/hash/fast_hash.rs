//! This module, when enabled with the `fast_hash` feature flag, replaces the Poseidon hash
//! implementation with `blake2b_simd`, which is much faster. **This is useless for production use
//! of this crate; it is only useful to accelerate testing, when we don't care about producing
//! zero-knowledge proofs.**

use std::sync::Arc;

use ark_ff::fields::PrimeField;
use dashmap::DashMap;
use decaf377::FieldExt;
use hash_hasher::HashBuildHasher;
use once_cell::sync::Lazy;
use poseidon377::Fq;

#[allow(clippy::type_complexity)]
static HASH_1_CACHE: Lazy<Arc<DashMap<(Fq, Fq), Fq, HashBuildHasher>>> =
    Lazy::new(|| Arc::new(DashMap::with_hasher(HashBuildHasher::default())));

pub fn hash_1(domain_separator: &Fq, value: Fq) -> Fq {
    *HASH_1_CACHE
        .entry((*domain_separator, value))
        .or_insert_with(|| {
            let mut state = blake2b_simd::State::new();
            state.update(&domain_separator.to_bytes());
            state.update(&value.to_bytes());
            Fq::from_le_bytes_mod_order(state.finalize().as_bytes())
        })
}

#[allow(clippy::type_complexity)]
static HASH_4_CACHE: Lazy<Arc<DashMap<(Fq, (Fq, Fq, Fq, Fq)), Fq, HashBuildHasher>>> =
    Lazy::new(|| Arc::new(DashMap::with_hasher(HashBuildHasher::default())));

pub fn hash_4(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq)) -> Fq {
    *HASH_4_CACHE
        .entry((*domain_separator, value))
        .or_insert_with(|| {
            let mut state = blake2b_simd::State::new();
            state.update(&domain_separator.to_bytes());
            state.update(&value.0.to_bytes());
            state.update(&value.1.to_bytes());
            state.update(&value.2.to_bytes());
            state.update(&value.3.to_bytes());
            Fq::from_le_bytes_mod_order(state.finalize().as_bytes())
        })
}
