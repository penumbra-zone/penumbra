use crate::prelude::*;

/// The frontier (rightmost) leaf in a frontier of a tree.
///
/// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to contain
/// the inserted item.
#[derive(Clone, Copy, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item> {
    item: Item,
}

impl<Item: GetHash> GetHash for Leaf<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.item.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.item.cached_hash()
    }
}

impl<Item: Height> Height for Leaf<Item> {
    type Height = Item::Height;
}

impl<Item: Focus> Frontier for Leaf<Item> {
    type Item = Item;

    #[inline]
    fn new(item: Self::Item) -> Self {
        Self { item }
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        Some(f(&mut self.item))
    }

    #[inline]
    fn focus(&self) -> Option<&Self::Item> {
        Some(&self.item)
    }

    #[inline]
    /// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to
    /// contain the inserted item.
    fn insert_owned(self, item: Self::Item) -> Result<Self, Full<Self>> {
        Err(Full {
            item,
            complete: self.finalize_owned(),
        })
    }

    #[inline]
    fn is_full(&self) -> bool {
        true
    }
}

impl<Item: Focus> Focus for Leaf<Item> {
    type Complete = complete::Leaf<<Item as Focus>::Complete>;

    #[inline]
    fn finalize_owned(self) -> Insert<Self::Complete> {
        self.item.finalize_owned().map(complete::Leaf::new)
    }
}

impl<Item: Witness> Witness for Leaf<Item> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        self.item.witness(index)
    }
}

impl<Item: GetPosition> GetPosition for Leaf<Item> {
    #[inline]
    fn position(&self) -> Option<u64> {
        self.item.position()
    }
}

impl<Item: GetHash + Forget> Forget for Leaf<Item> {
    #[inline]
    fn forget(&mut self, forgotten: Option<Forgotten>, index: impl Into<u64>) -> bool {
        self.item.forget(forgotten, index)
    }
}

impl<'tree, Item: GetPosition + Height + structure::Any<'tree>> structure::Any<'tree>
    for Leaf<Item>
{
    fn kind(&self) -> Kind {
        self.item.kind()
    }

    fn forgotten(&self) -> Forgotten {
        self.item.forgotten()
    }

    fn children(&'tree self) -> Vec<HashOrNode<'tree>> {
        self.item.children()
    }
}

impl<Item: OutOfOrder> OutOfOrder for Leaf<Item> {
    fn uninitialized(position: Option<u64>, forgotten: Forgotten) -> Self {
        Self {
            item: Item::uninitialized(position, forgotten),
        }
    }

    fn uninitialized_out_of_order_insert_commitment(&mut self, index: u64, commitment: Commitment) {
        self.item
            .uninitialized_out_of_order_insert_commitment(index, commitment)
    }
}

impl<Item: UncheckedSetHash> UncheckedSetHash for Leaf<Item> {
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        self.item.unchecked_set_hash(index, height, hash)
    }

    fn finish_initialize(&mut self) {
        self.item.finish_initialize()
    }
}
