use crate::{GetHash, Hash, Height};

pub struct Node<Child> {
    hash: Hash,
    children: Option<Box<[Child; 4]>>,
}

impl<Child> Node<Child> {
    /// This is a *dangerous function*: it does not check or recompute the hash given, so it should
    /// only be used when it is known that the hash is the correct hash of the children provided.
    pub(super) fn from_parts_unchecked(hash: Hash, children: Option<[Child; 4]>) -> Self
    where
        Child: GetHash + Height,
    {
        // Check that we're not violating the hash equality invariant, but only in debug mode
        debug_assert_eq!(
            hash,
            if let Some([ref a, ref b, ref c, ref d]) = children {
                Hash::node(Self::HEIGHT, a.hash(), b.hash(), c.hash(), d.hash())
            } else {
                hash
            }
        );
        Self {
            hash,
            children: children.map(Box::new),
        }
    }
}

impl<Child: Height> Height for Node<Child> {
    const HEIGHT: usize = Child::HEIGHT + 1;
}

impl<Child> GetHash for Node<Child> {
    fn hash(&self) -> Hash {
        self.hash
    }
}
