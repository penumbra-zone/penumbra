use crate::{GetHash, Hash};

pub struct Node<Child> {
    hash: Hash,
    children: Option<Box<[Child; 4]>>,
}

impl<Child> Node<Child> {
    /// This is a *dangerous function*: it does not check or recompute the hash given, so it should
    /// only be used when it is known that the hash is the correct hash of the children provided.
    pub(super) fn from_parts_unchecked(hash: Hash, children: Option<[Child; 4]>) -> Self {
        Self {
            hash,
            children: children.map(Box::new),
        }
    }
}

impl<Child> GetHash for Node<Child> {
    fn hash(&self) -> Hash {
        self.hash
    }
}
