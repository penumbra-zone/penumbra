#[macro_use]
extern crate derivative;

pub mod hash;
pub use hash::{GetHash, Hash};

pub mod item;
pub use item::Item;

pub use internal::height::Height;

pub mod internal;
pub use internal::Insert;
use internal::{Active, Complete, Focus, Full};

pub struct Commitment;

impl GetHash for Commitment {
    fn hash(&self) -> Hash {
        Hash
    }
}
