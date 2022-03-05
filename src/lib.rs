mod three;
use three::{Elems, Three};

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

    /// **Important:** If returning [`Inserted::Failure`], the returned `Self` *must be the same* as
    /// the original input. Violating this will break internal assumptions about the validity of
    /// hashes, because when a failure occurs, we do not re-hash the returned thing.
    fn insert(self, item: Self::Item) -> Inserted<Self>;

    fn witness(&mut self);

    fn complete(self) -> Self::Complete;
}

enum Inserted<T: Active> {
    Success(T),
    Full(T::Item, T::Complete),
    Failure(T::Item, T),
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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
