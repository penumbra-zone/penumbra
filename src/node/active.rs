use crate::{Elems, GetHash, Hash, Height, Three};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Active<Focus: crate::Active> {
    focus: Focus,
    siblings: Three<Focus::Complete>,
    hash: Hash,
}

impl<Focus: crate::Active> Active<Focus> {
    pub(crate) fn from_parts(siblings: Three<Focus::Complete>, focus: Focus) -> Self
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

fn hash_active<Sibling: GetHash, Focus: crate::Active + GetHash>(
    siblings: &Three<Sibling>,
    focus: &Focus,
) -> Hash {
    // Get the correct padding hash for this height
    let padding = Hash::padding();

    // Get the four elements of this segment, *in order*, and extract their hashes
    let (a, b, c, d) = match siblings.elems() {
        Elems::_0([]) => {
            let a = focus.hash();
            let [b, c, d] = [padding, padding, padding];
            (a, b, c, d)
        }
        Elems::_1(full) => {
            let [a] = full.map(Sibling::hash);
            let b = focus.hash();
            let [c, d] = [padding, padding];
            (a, b, c, d)
        }
        Elems::_2(full) => {
            let [a, b] = full.map(Sibling::hash);
            let c = focus.hash();
            let [d] = [padding];
            (a, b, c, d)
        }
        Elems::_3(full) => {
            let [a, b, c] = full.map(Sibling::hash);
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
    fn witness(&mut self) {
        self.focus.witness();
    }

    #[inline]
    fn complete(self) -> Self::Complete {
        super::Complete::from_siblings_and_focus_unchecked(
            self.hash,
            self.siblings,
            self.focus.complete(),
        )
    }

    #[inline]
    fn alter(&mut self, f: impl FnOnce(&mut Self::Item)) {
        self.focus.alter(f);
        self.hash = hash_active(&self.siblings, &self.focus);
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Self::Complete)> {
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
                    let node = super::Complete::from_parts_unchecked(hash, complete);
                    Err((item, node))
                }
            },
        }
    }
}
