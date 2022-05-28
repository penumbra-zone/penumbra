//! A dynamic representation of nodes within the tree structure, for writing homogeneous traversals.

use std::fmt::{Debug, Display};

use crate::prelude::*;

/// Every kind of node in the tree implements [`Any`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub trait Any: Versioned {
    /// The place this node is located: on the frontier or in the complete interior.
    fn place(&self) -> Place;

    /// The kind of node this is: an item at the base, a leaf of some tier, an internal node, a
    /// tier root, or a top-level root.
    fn kind(&self) -> Kind;

    /// The height of this node above the base of the tree.
    fn height(&self) -> u8;

    /// Whether or not this thing is finalized.
    fn finalized(&self) -> bool;

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    fn index(&self) -> u64 {
        0
    }

    /// The children, or hashes of them, of this node.
    fn children(&self) -> Vec<Insert<Child>>;

    /// The unique key describing this node in space and time.
    fn key(&self) -> Key {
        Key {
            version: self.version(),
            height: self.height(),
            kind: self.kind(),
            index: self.index(),
        }
    }

    /// The value associated with this node. Together with the node's key and the collection of all
    /// other nodes' keys and values, this allows the tree to be reconstructed.
    fn value(&self) -> Value {
        Value {
            finalized: self.finalized(),
            children: self
                .children()
                .into_iter()
                .map(|insert| insert.map(Version::version))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    pub version: Version,
    pub height: u8,
    pub kind: Kind,
    pub index: u64,
}

impl Key {
    pub fn child(&self, index: u64, version: Version) -> Self {
        todo!("calculate what the descendent key should be")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub finalized: bool,
    pub children: Vec<Insert<Version>>,
}

pub trait Reconstruct: Sized {
    fn reconstruct<R: Read>(reader: &R, version: Version, index: u64) -> Result<Self, R::Error>;
}

/*
    Table schema (all columns non-optional except as noted):

    TABLE latest:

        | latest_version |
        +----------------+
        | u64            |

    TABLE nodes:

        /---- composite primary key ------\
        | version | height | kind | index | finalized | hash          | marked_version |
        +---------+--------+------+-------+-----------+---------------+----------------+
        | u64     | u8     | text | u64   | bool      | optional blob | u64            |

    TABLE children:

        /---- composite primary key --------------\/--- exactly one is non-null --\
        /---- foreign keys ---------------\       |                               |
        | version | height | kind | index | child | child_version | child_hash    |
        +---------+--------+------+-------+-------+---------------+---------------+
        | u64     | u8     | text | u64   | u8    | optional u64  | optional blob |

    Other constraints:

    - kind is one of 'item', 'leaf', 'node', 'tier', or 'top'
    - height <= 24
    - if kind is 'item', height is 0
    - if kind is 'leaf', height is 0, 8, or 16
    - if kind is 'node', height is > 0
    - if kind is 'tier', height is 8, 16, or 24
    - if kind is 'top', height is 24
    - index < 4^(24 - height)
*/

// TODO: async
pub trait Read {
    type Error;

    // get the latest version stored
    fn latest(&self) -> Result<Version, Self::Error>;

    // get the value associated with this key (returns error if missing key)
    fn get(&self, key: Key) -> Result<Value, Self::Error>;

    // get the cached hash, if any
    fn hash(&self, key: Key) -> Result<Option<Hash>, Self::Error>;
}

// TODO: async
pub trait Write: Read {
    type Error: From<<Self as Read>::Error>;

    // should error on trying to overwrite a key if the value is different
    // created entries are automatically marked with the value of their own key
    fn create(&mut self, key: Key, value: Value) -> Result<(), <Self as Write>::Error>;

    // should error on trying to overwrite a hash that's already cached and is different
    fn cache(&mut self, key: Key, hash: Hash) -> Result<(), <Self as Write>::Error>;

    // mark this key as to-be-preserved up to the latest version (not recursive)
    fn mark(&mut self, key: Key) -> Result<(), <Self as Write>::Error>;

    // delete any key if its marked version is strictly less than the latest one
    fn sweep(&mut self) -> Result<(), <Self as Write>::Error>;
}

fn debug_any(this: &dyn Any, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f
        .debug_struct(&format!("{}::{}", this.place(), this.kind()))
        .field("version", &u64::from(this.version()))
        .field("height", &this.height())
        .field("index", &this.index());
    if let Some(hash) = this.cached_hash() {
        s = s.field("hash", &hash);
    }
    s.field("finalized", &this.finalized())
        .field("children", &this.children())
        .finish()
}

fn display_any(this: &dyn Any, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(&format!("{}::{}", this.place(), this.kind()))
        .field("version", &u64::from(this.version()))
        .field("height", &this.height())
        .field("index", &this.index())
        .finish_non_exhaustive()
}

impl Debug for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_any(self, f)
    }
}

impl Display for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_any(self, f)
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// An item node at the bottom of the tree.
    Item,
    /// A leaf node at the bottom of some tier.
    Leaf,
    /// An internal node within some tier.
    Node,
    /// The root of a tier node.
    Tier,
    /// The top of a tree.
    Top,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Item => write!(f, "Item"),
            Kind::Leaf => write!(f, "Leaf"),
            Kind::Node => write!(f, "Node"),
            Kind::Tier => write!(f, "Tier"),
            Kind::Top => write!(f, "Top"),
        }
    }
}

/// The place a node is located in a tree: whether it is on the frontier or is completed.
///
/// This is redundant with the pair of (height, index) if the total size of the tree is known, but
/// it is useful to reveal it directly.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// The node is not on the frontier.
    Complete,
    /// The node is on the frontier.
    Frontier,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Frontier => write!(f, "frontier"),
            Place::Complete => write!(f, "complete"),
        }
    }
}

/// A child of an [`Any`]: this implements [`Any`] and supertraits, so can and should be treated
/// equivalently.
pub struct Child<'a> {
    offset: u64,
    inner: &'a dyn Any,
}

impl Debug for Child<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_any(self, f)
    }
}

impl Display for Child<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_any(self, f)
    }
}

impl<'a> Child<'a> {
    /// Make a new [`Child`] from a reference to something implementing [`Any`].
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

impl Versioned for Child<'_> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn hash_version(&self) -> Option<Version> {
        self.inner.hash_version()
    }

    fn set_version(&mut self, version: Version) {
        self.inner.set_version(version)
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
                child.map(|child| {
                    debug_assert_eq!(
                        child.offset, 0,
                        "explicitly constructed children should have zero offset"
                    );
                    // If the height doesn't change, we shouldn't be applying a multiplier to the
                    // parent offset:
                    let multiplier = 4u64.pow((self.height() - child.height()).into());
                    Child {
                        inner: child.inner,
                        offset: self.offset * multiplier + nth as u64,
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indexing_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new();
        for i in 0..=MAX_SIZE_TO_TEST {
            top.insert(Commitment(i.into()).into()).unwrap();
        }

        fn check_leaves(index: &mut [[u64; 5]; 9], node: &dyn Any) {
            assert_eq!(
                node.index(),
                index[usize::from(node.height())][node.kind() as usize],
                "{}",
                node
            );

            index[usize::from(node.height())][node.kind() as usize] += 1;

            for child in node
                .children()
                .iter()
                .filter_map(|child| child.as_ref().keep())
            {
                check_leaves(index, child);
            }
        }

        check_leaves(&mut [[0; 5]; 9], &top);
    }
}
