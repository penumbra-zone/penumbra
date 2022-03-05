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
    pub(super) fn from_siblings_and_focus_unchecked(
        hash: Hash,
        siblings: Three<Child>,
        focus: Child,
    ) -> Self
    where
        Child: crate::Complete,
    {
        // Place the focus in the appropriate position in the children array
        let children = match siblings.into_elems() {
            IntoElems::_0([]) => [Some(focus), None, None, None],
            IntoElems::_1([a]) => [Some(a), Some(focus), None, None],
            IntoElems::_2([a, b]) => [Some(a), Some(b), Some(focus), None],
            IntoElems::_3([a, b, c]) => [Some(a), Some(b), Some(c), Some(focus)],
        };
        // Filter the children array to prune out unwitnessed subtrees
        let children =
            children.map(|child| child.filter(Child::witnessed).map(|child| Box::new(child)));
        // Return the node
        Self { hash, children }
    }

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
