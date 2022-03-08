//! The tiered commitment tree for Penumbra.

#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

pub mod hash;
#[doc(inline)]
pub use hash::{GetHash, Hash};

pub mod item;
#[doc(inline)]
pub use item::Item;

#[doc(inline)]
pub use internal::height::Height;

pub mod internal;
#[doc(inline)]
pub use internal::active::Insert;
use internal::{
    active::{Active, Focus, Full},
    complete::Complete,
};

/// A commitment stored in the tree.
pub struct Commitment;

impl GetHash for Commitment {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::commitment(self)
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        None
    }
}
