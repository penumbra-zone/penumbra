use crate::prelude::*;

pub trait Any: GetHash {
    fn place(&self) -> Place;

    fn kind(&self) -> Kind;

    fn height(&self) -> u8;

    fn index(&self) -> u64 {
        0
    }

    fn children(&self) -> Vec<Insert<Child>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Complete,
    Frontier,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Item,
    Leaf,
    Node,
    Tier,
    Top,
}

pub struct Child<'a> {
    offset: u64,
    inner: &'a dyn Any,
}

impl<'a> Child<'a> {
    pub fn new(child: &'a dyn Any) -> Self {
        Child {
            offset: 0,
            inner: child,
        }
    }
}

impl GetHash for Child<'_> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl Any for Child<'_> {
    fn place(&self) -> Place {
        self.inner.place()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn height(&self) -> u8 {
        self.inner.height()
    }

    fn index(&self) -> u64 {
        self.offset + self.inner.index()
    }

    fn children(&self) -> Vec<Insert<Child>> {
        self.inner
            .children()
            .into_iter()
            .enumerate()
            .map(|(nth, child)| {
                child.map(|child| Child {
                    inner: child.inner,
                    offset: self.offset * 4 + child.offset + nth as u64,
                })
            })
            .collect()
    }
}
