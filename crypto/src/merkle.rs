use std::convert::{TryFrom, TryInto};

use ark_ff::{PrimeField, Zero};
use decaf377::{FieldExt, Fq};
use incrementalmerkletree;
pub use incrementalmerkletree::{
    bridgetree::{self, AuthFragment, BridgeTree},
    Altitude, Frontier, Hashable, Position, Recording, Tree,
};
use once_cell::sync::Lazy;

use crate::note;

pub const DEPTH: usize = 32;
pub type NoteCommitmentTree = BridgeTree<note::Commitment, { DEPTH as u8 }>;

/// The domain separator used to hash items into the Merkle tree.
pub static MERKLE_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.merkle.tree").as_bytes())
});

// Return value from `Tree::authentication_path(value: &note::Commitment)`
pub type Path = (Position, Vec<note::Commitment>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root(pub Fq);

impl TryFrom<&[u8]> for Root {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into()?;

        let inner = Fq::from_bytes(bytes)?;

        Ok(Root(inner))
    }
}

impl Root {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
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

#[cfg(feature = "sqlx")]
mod sqlx_impls {
    use sqlx::{Database, Decode, Encode, Postgres, Type};

    use super::*;

    impl<'r> Decode<'r, Postgres> for Root {
        fn decode(
            value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let bytes = Vec::<u8>::decode(value)?;
            Root::try_from(&bytes[..]).map_err(Into::into)
        }
    }

    impl<'q> Encode<'q, Postgres> for Root {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            let bytes = self.to_bytes();
            (&bytes[..]).encode(buf)
        }
    }

    impl Type<Postgres> for Root {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            <[u8] as Type<Postgres>>::type_info()
        }
    }
}
