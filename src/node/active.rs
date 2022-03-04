use crate::{GetHash, Hash, Height, Split, Three};

pub struct Active<Sibling, Focus> {
    focus: Focus,
    siblings: Three<Sibling>,
    witnessed: bool,
    hash: Hash,
}

impl<Sibling, Focus> Active<Sibling, Focus> {
    pub(crate) fn from_parts(siblings: Three<Sibling>, focus: Focus) -> Self
    where
        Focus: crate::Active + GetHash,
        Sibling: GetHash,
    {
        // Get the four elements of this segment, *in order*, and extract their hashes
        let (a, b, c, d) = match siblings.split() {
            Split::_0([], default) => {
                let a = focus.hash();
                let [b, c, d] = default.map(Sibling::hash);
                (a, b, c, d)
            }
            Split::_1(full, default) => {
                let [a] = full.map(Sibling::hash);
                let b = focus.hash();
                let [c, d] = default.map(Sibling::hash);
                (a, b, c, d)
            }
            Split::_2(full, default) => {
                let [a, b] = full.map(Sibling::hash);
                let c = focus.hash();
                let [d] = default.map(Sibling::hash);
                (a, b, c, d)
            }
            Split::_3(full, []) => {
                let [a, b, c] = full.map(Sibling::hash);
                let d = focus.hash();
                (a, b, c, d)
            }
        };

        let hash = Hash::node(Focus::HEIGHT + 1, a, b, c, d);
        Self {
            witnessed: false,
            hash,
            siblings,
            focus,
        }
    }
}

impl<Sibling, Focus> Height for Active<Sibling, Focus>
where
    Sibling: Height,
    Focus: Height,
{
    const HEIGHT: usize = if Focus::HEIGHT == Sibling::HEIGHT {
        Focus::HEIGHT + 1
    } else {
        panic!("`Sibling::HEIGHT` does not match `Focus::HEIGHT` in `Segment`: check for improper depth types")
    };
}

impl<Sibling, Focus> crate::Active for Active<Sibling, Focus>
where
    Focus: crate::Active<Complete = Sibling> + GetHash,
    Sibling: crate::Complete<Active = Focus> + Height + GetHash + Default,
{
    type Item = Focus::Item;
    type Complete = super::Complete<Sibling>;

    fn singleton(item: Self::Item) -> Self {
        let focus = Focus::singleton(item);
        let siblings = Three::new();
        Self::from_parts(siblings, focus)
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Self::Complete)> {
        let Active {
            witnessed,
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
                    let node = super::Complete::from_parts_unchecked(
                        // We can avoid recomputing this hash because our hash calculation is
                        // carefully designed to hash in the exact same order as the hash
                        // calculation for a node itself
                        hash,
                        // If this segment was not marked as witnessed, we know that any
                        // sub-segments are likewise not witnessed, so we can erase the subtree
                        if witnessed { Some(complete) } else { None },
                    );
                    Err((item, node))
                }
            },
        }
    }
}
