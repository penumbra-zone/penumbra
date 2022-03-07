use crate::{Elems, GetHash, Hash, Height, Three};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Active<Focus: crate::Active> {
    focus: Focus,
    siblings: Three<Result<Focus::Complete, Hash>>,
    hash: Hash,
}

impl<Focus: crate::Active> Active<Focus> {
    pub(crate) fn from_parts(siblings: Three<Result<Focus::Complete, Hash>>, focus: Focus) -> Self
    where
        Focus: crate::Active + GetHash,
    {
        Self {
            hash: hash_active(&siblings, &focus),
            siblings,
            focus,
        }
    }
}

fn hash_active<Focus: crate::Active + GetHash>(
    siblings: &Three<Result<Focus::Complete, Hash>>,
    focus: &Focus,
) -> Hash {
    // Get the correct padding hash for this height
    let padding = Hash::padding();

    // Get the hashes of all the `Result<T, Hash>` in the array, hashing `T` as necessary
    #[inline]
    fn hashes_of_all<T: GetHash, const N: usize>(full: [&Result<T, Hash>; N]) -> [Hash; N] {
        full.map(|result| {
            result
                .as_ref()
                .map(|complete| complete.hash())
                .unwrap_or_else(|hash| *hash)
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

    Hash::node(Focus::HEIGHT + 1, a, b, c, d)
}

impl<Focus: crate::Active> Height for Active<Focus>
where
    Focus: Height,
{
    const HEIGHT: usize = if Focus::HEIGHT == <Focus as crate::Active>::Complete::HEIGHT {
        Focus::HEIGHT + 1
    } else {
        panic!("`Sibling::HEIGHT` does not match `Focus::HEIGHT` in `Segment`: check for improper depth types")
    };
}

impl<Focus: crate::Active> GetHash for Active<Focus> {
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl<Focus> crate::Active for Active<Focus>
where
    Focus: crate::Active + GetHash,
{
    type Item = Focus::Item;
    type Complete = super::Complete<Focus::Complete>;

    #[inline]
    fn singleton(item: Self::Item) -> Self {
        let focus = Focus::singleton(item);
        let siblings = Three::new();
        Self::from_parts(siblings, focus)
    }

    #[inline]
    fn complete(self) -> Result<Self::Complete, Hash> {
        super::Complete::try_from_siblings_and_focus_or_else_hash(
            self.siblings,
            self.focus.complete(),
        )
    }

    #[inline]
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        let result = self.focus.alter(f);
        self.hash = hash_active(&self.siblings, &self.focus);
        result
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Result<Self::Complete, Hash>)> {
        let Active {
            focus,
            siblings,
            hash, // NOTE: ONLY VALID TO RE-USE WHEN CONSTRUCTING A NODE
        } = self;

        match focus.insert(item) {
            // We successfully inserted at the focus, so siblings don't need to be changed
            Ok(focus) => Ok(Self::from_parts(siblings, focus)),

            // We couldn't insert at the focus because it was full, so we need to move our path
            // rightwards and insert into a newly created focus
            Err((item, sibling)) => match siblings.push(sibling) {
                // We had enough room to add another sibling, so we set our focus to a new focus
                // containing only the item we couldn't previously insert
                Ok(siblings) => {
                    let focus = Focus::singleton(item);
                    Ok(Self::from_parts(siblings, focus))
                }
                // We didn't have enough room to add another sibling, so we return a complete node
                // as a carry, to be propagated up above us and added to some ancestor segment's
                // siblings, along with the item we couldn't insert
                Err(complete) => {
                    // This is okay because `complete` is guaranteed to have the same elements in
                    // the same order as `siblings + [focus]`.
                    Err((
                        item,
                        super::Complete::try_from_children(complete).ok_or(hash),
                    ))
                }
            },
        }
    }
}
