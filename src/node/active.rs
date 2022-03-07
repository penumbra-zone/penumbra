use std::cell::Cell;

use crate::{
    height::{IsHeight, S},
    Elems, Full, GetHash, Hash, HashOr, Height, Three,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Active<Focus: crate::Active> {
    focus: Focus,
    siblings: Three<HashOr<Focus::Complete>>,
    // TODO: replace this with space-saving `Cell<OptionHash>`?
    hash: Cell<Option<Hash>>,
}

impl<Focus: crate::Active> Active<Focus> {
    pub(crate) fn from_parts(siblings: Three<HashOr<Focus::Complete>>, focus: Focus) -> Self
    where
        Focus: crate::Active + GetHash,
    {
        Self {
            hash: Cell::new(None),
            siblings,
            focus,
        }
    }
}

fn hash_active<Focus: crate::Active + GetHash>(
    siblings: &Three<HashOr<Focus::Complete>>,
    focus: &Focus,
) -> Hash {
    // Get the correct padding hash for this height
    let padding = Hash::padding();

    // Get the hashes of all the `Result<T, Hash>` in the array, hashing `T` as necessary
    #[inline]
    fn hashes_of_all<T: GetHash, const N: usize>(full: [&HashOr<T>; N]) -> [Hash; N] {
        full.map(|hash_or_t| match hash_or_t {
            HashOr::Hash(hash) => *hash,
            HashOr::Item(t) => t.hash(),
        })
    }

    // Get the four elements of this segment, *in order*, and extract their hashes
    let (a, b, c, d) = match siblings.elems() {
        Elems::_0([]) => {
            let a = focus.hash();
            let [b, c, d] = [padding, padding, padding];
            (a, b, c, d)
        }
        Elems::_1(full) => {
            let [a] = hashes_of_all(full);
            let b = focus.hash();
            let [c, d] = [padding, padding];
            (a, b, c, d)
        }
        Elems::_2(full) => {
            let [a, b] = hashes_of_all(full);
            let c = focus.hash();
            let [d] = [padding];
            (a, b, c, d)
        }
        Elems::_3(full) => {
            let [a, b, c] = hashes_of_all(full);
            let d = focus.hash();
            (a, b, c, d)
        }
    };

    Hash::node(Focus::Height::HEIGHT + 1, a, b, c, d)
}

impl<Focus: crate::Active> Height for Active<Focus>
where
    Focus: Height,
{
    type Height = S<Focus::Height>;
}

impl<Focus: crate::Active> GetHash for Active<Focus> {
    #[inline]
    fn hash(&self) -> Hash {
        match self.hash.get() {
            Some(hash) => hash,
            None => {
                let hash = hash_active(&self.siblings, &self.focus);
                self.hash.set(Some(hash));
                hash
            }
        }
    }
}

impl<Focus: crate::Active> crate::Finalize for Active<Focus> {
    type Complete = super::Complete<Focus::Complete>;

    #[inline]
    fn finalize(self) -> HashOr<Self::Complete> {
        super::Complete::from_siblings_and_focus_or_else_hash(self.siblings, self.focus.finalize())
    }
}

impl<Focus> crate::Active for Active<Focus>
where
    Focus: crate::Active + GetHash,
{
    type Item = Focus::Item;

    #[inline]
    fn singleton(item: HashOr<Self::Item>) -> Self {
        let focus = Focus::singleton(item);
        let siblings = Three::new();
        Self::from_parts(siblings, focus)
    }

    #[inline]
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        let result = self.focus.alter(f);
        if result.is_some() {
            // If the function was run, clear the cached hash, because it could now be invalid
            self.hash.set(None);
        }
        result
    }

    #[inline]
    fn insert(self, item: HashOr<Self::Item>) -> Result<Self, Full<Self::Item, Self::Complete>> {
        match self.focus.insert(item) {
            // We successfully inserted at the focus, so siblings don't need to be changed
            Ok(focus) => Ok(Self::from_parts(self.siblings, focus)),

            // We couldn't insert at the focus because it was full, so we need to move our path
            // rightwards and insert into a newly created focus
            Err(Full {
                item,
                complete: sibling,
            }) => match self.siblings.push(sibling) {
                // We had enough room to add another sibling, so we set our focus to a new focus
                // containing only the item we couldn't previously insert
                Ok(siblings) => Ok(Self::from_parts(siblings, Focus::singleton(item))),

                // We didn't have enough room to add another sibling, so we return a complete node
                // as a carry, to be propagated up above us and added to some ancestor segment's
                // siblings, along with the item we couldn't insert
                Err(children) => {
                    Err(Full {
                        item,
                        // Implicitly, we only hash the children together when we're pruning them
                        // (because otherwise we would lose that informtion); if at least one child
                        // and its sibling hashes/subtrees is preserved in a `Complete` node, then
                        // we defer calculating the node hash until looking up an authentication path
                        complete: match super::Complete::from_children_or_else_hash(children) {
                            HashOr::Hash(hash) => HashOr::Hash(hash),
                            HashOr::Item(node) => {
                                if let Some(hash) = self.hash.get() {
                                    // This is okay because `complete` is guaranteed to have the same elements in
                                    // the same order as `siblings + [focus]`:
                                    node.set_hash_unchecked(hash);
                                }
                                HashOr::Item(node)
                            }
                        },
                    })
                }
            },
        }
    }
}
