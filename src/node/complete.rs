use crate::{
    three::{IntoElems, Three},
    GetHash, Hash, HashOr, Height,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Complete<Child> {
    hash: Hash,
    children: [Option<Box<Child>>; 4],
}

impl<Child> Complete<Child> {
    /// Only call this when you know what the hash should be!
    pub(super) fn set_hash_unchecked(&self, hash: Hash) {
        todo!("set the hash");
    }

    pub(super) fn from_siblings_and_focus_or_else_hash(
        siblings: Three<HashOr<Child>>,
        focus: HashOr<Child>,
    ) -> HashOr<Self>
    where
        Child: crate::Complete,
    {
        todo!("construct `Complete` from siblings and focus")
    }

    pub(super) fn from_children_or_else_hash(children: [HashOr<Child>; 4]) -> HashOr<Self>
    where
        Child: crate::Complete + GetHash + Height,
    {
        todo!("construct `Complete` from all four children")
    }
}

impl<Child: Height> Height for Complete<Child> {
    const HEIGHT: usize = Child::HEIGHT + 1;
}

impl<Child: crate::Complete> crate::Complete for Complete<Child> {
    type Active = super::Active<Child::Active>;
}

impl<Child> GetHash for Complete<Child> {
    fn hash(&self) -> Hash {
        self.hash
    }
}
