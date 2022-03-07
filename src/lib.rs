#[macro_use]
extern crate derivative;

mod three;
use three::{Elems, Three};

pub mod leaf;
pub mod level;
pub mod node;

trait Height {
    const HEIGHT: usize;
}

trait Active: Height + GetHash + Sized {
    type Item;
    type Complete: Complete<Active = Self>;

    fn singleton(item: Self::Item) -> Self;

    #[allow(clippy::type_complexity)]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Result<Self::Complete, Hash>)>;

    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T>;

    fn complete(self) -> Result<Self::Complete, Hash>;
}

trait Complete: Height + GetHash {
    type Active: Active<Complete = Self>;

    fn witnessed(&self) -> bool;
}

trait GetHash {
    fn hash(&self) -> Hash;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hash;

#[allow(unused)]
impl Hash {
    fn padding() -> Hash {
        Hash
    }

    fn leaf(height: usize, commitment: &Commitment) -> Hash {
        Hash
    }

    fn node(height: usize, a: Hash, b: Hash, c: Hash, d: Hash) -> Hash {
        Hash
    }
}

pub struct Commitment;

impl GetHash for Commitment {
    fn hash(&self) -> Hash {
        Hash
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
