use crate::{
    three::{IntoElems, Three},
    GetHash, Hash, Height,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Complete<Child> {
    hash: Hash,
    children: [Option<Box<Child>>; 4],
}

impl<Child> Complete<Child> {
    /// This is a *dangerous function*: it does not check or recompute the hash given, so it should
    /// only be used when it is known that the hash is the correct hash of the children provided.
    pub(super) fn try_from_siblings_and_focus_or_else_hash(
        siblings: Three<Result<Child, Hash>>,
        focus: Result<Child, Hash>,
    ) -> Result<Self, Hash>
    where
        Child: crate::Complete,
    {
        todo!("construct `Complete` from siblings and focus")
    }

    /// This is a *dangerous function*: it does not check or recompute the hash given, so it should
    /// only be used when it is known that the hash is the correct hash of the children provided.
    pub(super) fn try_from_children(children: [Result<Child, Hash>; 4]) -> Option<Self>
    where
        Child: crate::Complete + GetHash + Height,
    {
        todo!("construct `Complete` from all four children")
    }
}

impl<Child: Height> Height for Complete<Child> {
    const HEIGHT: usize = Child::HEIGHT + 1;
}

impl<Child> crate::Complete for Complete<Child>
where
    Child: crate::Complete + GetHash,
    Child::Active: GetHash,
{
    type Active = super::Active<Child::Active>;

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
