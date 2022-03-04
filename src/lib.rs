mod three;
use three::{Split, Three};

mod leaf;
pub use leaf::Leaf;

pub mod level;
pub mod node;

trait Height {
    const HEIGHT: usize;
}

trait Active: Height + Sized {
    type Item;
    type Complete: Complete<Active = Self>;

    fn singleton(item: Self::Item) -> Self;

    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Self::Complete)>;

    fn witness(&mut self);
}

trait Complete: Height {
    type Active: Active<Complete = Self>;

    fn witnessed(&self) -> bool;
}

trait GetHash {
    fn hash(&self) -> Hash;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hash;

impl Hash {
    fn leaf(height: usize, commitment: &Commitment) -> Hash {
        Hash
    }

    fn node(height: usize, a: Hash, b: Hash, c: Hash, d: Hash) -> Hash {
        Hash
    }
}

pub struct Commitment;

type X = node::Active<Leaf<0>, Leaf<0>>;

const Y: usize = X::HEIGHT;
