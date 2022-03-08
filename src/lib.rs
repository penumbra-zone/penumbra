#[macro_use]
extern crate derivative;

mod three;
use three::{Elems, Three};

pub mod hash;
pub use hash::{GetHash, Hash, HashOr};

pub mod item;
pub use item::Item;

pub mod height;
pub use height::Height;

pub mod internal;
use internal::{Active, Complete, Focus, Full};

pub struct Commitment;

impl GetHash for Commitment {
    fn hash(&self) -> Hash {
        Hash
    }
}
