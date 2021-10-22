use ark_ff::PrimeField;
use ark_ff::Zero;
use decaf377::{FieldExt, Fq};
use incrementalmerkletree;
use once_cell::sync::Lazy;
use std::convert::{TryFrom, TryInto};

use crate::note;

pub use incrementalmerkletree::{
    bridgetree::{self, AuthFragment, BridgeTree},
    Altitude, Hashable, Position, Recording, Tree,
};

pub const MERKLE_DEPTH: usize = 32;

/// The domain separator used to hash items into the Merkle tree.
pub static MERKLE_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.merkle.tree").as_bytes())
});

// Return value from `Tree::authentication_path(value: &note::Commitment)`
pub type Path = (usize, Vec<note::Commitment>);

#[derive(PartialEq, Eq)]
pub struct Root(pub Fq);

impl TryFrom<&[u8]> for Root {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into()?;

        let inner = Fq::from_bytes(bytes)?;

        Ok(Root(inner))
    }
}

pub trait TreeExt {
    fn root2(&self) -> Root;
}

impl<T> TreeExt for T
where
    T: Tree<note::Commitment>,
{
    fn root2(&self) -> Root {
        Root(self.root().0)
    }
}

impl Hashable for note::Commitment {
    fn empty_leaf() -> Self {
        note::Commitment(Fq::zero())
    }

    fn combine(level: Altitude, a: &Self, b: &Self) -> Self {
        // extend to build domain sep
        let level_fq: Fq = u8::from(level).into();
        let level_domain_sep: Fq = *MERKLE_DOMAIN_SEP + level_fq;
        note::Commitment(poseidon377::hash_2(&level_domain_sep, (a.0, b.0)))
    }
}
