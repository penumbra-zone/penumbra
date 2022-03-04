use crate::{GetHash, Hash, Height};

pub struct Complete<Child> {
    hash: Hash,
    children: [Option<Box<Child>>; 4],
}

impl<Child> Complete<Child> {
    /// This is a *dangerous function*: it does not check or recompute the hash given, so it should
    /// only be used when it is known that the hash is the correct hash of the children provided.
    pub(super) fn from_parts_unchecked(hash: Hash, children: [Child; 4]) -> Self
    where
        Child: crate::Complete + GetHash + Height,
    {
        Self {
            hash,
            // Drop children which contain no witnesses
            children: children.map(|child| {
                if child.witnessed() {
                    Some(Box::new(child))
                } else {
                    None
                }
            }),
        }
    }
}

impl<Child: Height> Height for Complete<Child> {
    const HEIGHT: usize = Child::HEIGHT + 1;
}

impl<Child> crate::Complete for Complete<Child>
where
    Child: crate::Complete + GetHash + Default,
    Child::Active: GetHash,
{
    type Active = super::Active<Child, Child::Active>;

    #[inline]
    fn witnessed(&self) -> bool {
        self.children.iter().any(|child| child.is_some())
    }
}

impl<Child> GetHash for Complete<Child> {
    fn hash(&self) -> Hash {
        self.hash
    }
}
