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
pub use internal::Insert;
use internal::{Active, Complete, Focus, Full};

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
