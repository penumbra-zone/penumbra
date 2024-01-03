//! The internal implementation of the tree, exposed here for documentation.
//!
//! This module and its submodules should not be expected to follow semantic versioning.
//!
//! ## Structure of Implementation
//!
//! The tiered commitment tree is not accessed directly _as a tree_; rather, the
//! [`Tree`](crate::Tree), [`epoch::Builder`](crate::builder::epoch::Builder),
//! [`epoch::Finalized`](crate::builder::epoch::Finalized),
//! [`block::Builder`](crate::builder::block::Builder), and
//! [`block::Finalized`](crate::builder::block::Finalized) structs from the top level of the crate
//! contain a tree together with a hashmap which maps commitments to their corresponding index
//! within the tree. This `internal` module and all its submodules concern themselves solely with
//! the implementation of the tree itself, wherein commitments and their authentication paths are
//! accessed by index. The surrounding pieces of the crate make use of the internal-facing API
//! exposed by this module to implement an external API specific to the three-tiered
//! tree/epoch/block commitment tree required by Penumbra.
//!
//! The tiered commitment tree has a very specific structure, and in this implementation we make
//! strong Rust's type system to enforce that structure. In particular, we ensure that internal
//! nodes are non-empty, and that leaves occur only at the bottom-most level of the tree. A
//! consequence of this is that it is unrepresentable to build a tree which fails to be pruned of
//! internal nodes holding no witnessed leaves, or whose leaves are of mismatched depth. A lesser
//! benefit of this design is that recursive methods on trees can be monomorphized and therefore
//! potentially inlined to the height of tree they are called upon, avoiding ever performing runtime
//! checks about the tree structure (there no need to check whether a particular part of the tree is
//! a leaf, internal node, etc., because this is statically known).
//!
//! This structural guarantee is achieved by defining these tree structures as _nested generic
//! types_, and defining their methods as _recursive trait implementations_ (rather than as
//! inductive non-generic enumeration types with recursive methods). These traits are all defined in
//! [`interface`], but they are re-exported from [`frontier`] and [`complete`] as relevant.
//!
//! ## Tiers of Nodes of Leaves of Items: Frontier and Complete
//!
//! The primary exports of this module is the type [`frontier::Tier`]. It is in terms of this type
//! that the [`Tree`](crate::Tree) and [`builder`](crate::builder) structs are defined: a
//! [`Tree`](crate::Tree) is a `Top<Tier<Tier<Item>>>`, an epoch is a `Top<Tier<Item>>`, and a
//! `Block` is a `Top<Item>` (each with a managed index of commitments alongside).
//!
//! Internally, a [`Tier`](frontier::Tier) is a quadtree where every internal node is annotated with
//! the hash of its children, into which leaves (all at depth 8) are inserted in left to right
//! order. The "frontier" represents the path from the root to the rightmost leaf, at every level
//! containing any leftward siblings of a given frontier node, each of which is a [`complete`] tree
//! which stores the finalized bulk of the items inserted into the tree. As new leaves are created,
//! the frontier zig-zags rightwards, pushing finalized portions of itself into the leftward
//! complete tree.
//!
//! All [`Tier`](frontier::Tier)s must contain at least one child, and may be either unfinalized or
//! finalized; a [`Top`](frontier::Top) is like a tier, but it may be empty, and may not be
//! finalized. Stacking a [`Top`](frontier::Top) on top of [`Tier`](frontier::Tier)s allows there to
//! be a canonical representation for empty trees, and prevents the illegal state of a finalized
//! top-level tree.
//!
//! As described above, a variety of recursively defined traits are used to define the behavior of
//! trees. The [`Frontier`](frontier::Frontier) trait defines the operations possible on a frontier
//! of a tree, while the [`Focus`](frontier::Focus) trait defines how to operate over the tip of a
//! frontier, and the [`Forget`](frontier::Forget) trait defines how to remove witnessed leaves from
//! a frontier.
//!
//! While the frontier changes on every inserted leaf, within the complete portion of the tree the
//! only changes that occur are when leaves are forgotten and their containing nodes pruned. As a
//! result, the traits exposed by the [`complete`] module are merely
//! [`Complete`](complete::Complete), which is a marker trait used to ensure that every
//! [`Frontier`](frontier::Frontier) type is paired with a unique corresponding type of complete
//! tree, and the [`ForgetOwned`](complete::ForgetOwned) trait, which defines an equivalent to
//! [`frontier::Forget`] that is applicable to the by-value usage pattern of complete trees.

pub mod hash;
pub mod height;
pub mod interface;
pub mod path;
pub mod proof;
pub mod three;

mod insert;

#[allow(missing_docs)]
pub mod frontier {
    //! [`Frontier`] things can be inserted into and updated, always representing the rightmost
    //! (most recently inserted) element of a tree.
    //!
    //! In sketch: the structure of a single [`Tier`] contains eight [`Node`]s, the bottom-most of
    //! which contains a [`Leaf`]. Alternatively, a tier can be a summarized [`Hash`] of what its
    //! contents _would be_, and contain nothing at all besides this hash.
    //!
    //! At every level of a [`frontier::Tier`](Tier), the rightmost child of a
    //! [`frontier::Node`](Node) is a [`frontier::Node`](Node); all leftward siblings are
    //! [`complete::Node`](super::complete::Node)s. When the child of a [`frontier::Node`](Node)
    //! becomes entirely full (all its possible leaves are inserted), it is transformed into a
    //! [`complete::Node`](super::complete::Node) and appended to the list of complete siblings of
    //! its parent, thus shifting the frontier rightwards.
    //!
    //! At any given time, the frontier is always fully materialized; no node within it is ever
    //! summarized as a hash. It is at the point when a [`frontier::Node`](Node) becomes full and is
    //! then finalized into a [`complete::Node`](super::complete::Node) that it is pruned, if it
    //! contains no witnessed children.
    //!
    //! At the tip of the frontier, however deeply nested (perhaps within muliple [`Tier`]s), there
    //! is a single [`Item`], which is either a [`Commitment`](crate::Commitment) or a hash of one.
    //! Commitments can be inserted either with the intent to remember them, or with the intent to
    //! immediately forget them; this determines whether the [`Item`] is a commitment or merely its
    //! hash.
    pub(crate) use super::interface::OutOfOrder;
    #[doc(inline)]
    pub use super::interface::{Focus, Forget, Frontier, Full, GetPosition};
    pub(super) mod item;
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub(super) mod top;
    pub use super::insert::{Insert, InsertMut};
    #[doc(inline)]
    pub use {
        item::Item,
        leaf::Leaf,
        node::Node,
        tier::Tier,
        top::{Top, TrackForgotten},
    };
}

#[allow(missing_docs)]
pub mod complete {
    //! [`Complete`] things are sparse representations of only the data that was inserted using
    //! [`Witness::Keep`](crate::Witness::Keep), with the data that was inserted using
    //! [`Witness::Forget`](crate::Witness::Forget) being pruned eagerly.
    //!
    //! The structure of a single [`Tier`] contains eight levels of [`Node`]s, the bottom-most level
    //! of which contains [`Leaf`]s. Alternatively, a tier can be a summarized [`Hash`] of what its
    //! contents _would be_, and contain nothing at all besides this hash.
    //!
    //! In the internal levels of a [`complete::Tier`](Tier) are eight levels of
    //! [`complete::Node`](Node)s, each of which may have between one and four children. If a node
    //! does not have a given child, then it instead stores the hash that child used to have, when
    //! it existed. Empty nodes (all of whose children would be hashes) are unrepresentable, and
    //! instead their own hash is immediately stored in their parent node when their last child is
    //! forgotten.
    //!
    //! At the bottom of the bottom-most tier (perhaps at the bottom of multiple [`Tier`]s), there
    //! are [`Item`]s, each of which is merely a wrapper for a single
    //! [`Commitment`](crate::Commitment).
    pub(crate) use super::interface::OutOfOrderOwned;
    #[doc(inline)]
    pub use super::interface::{Complete, ForgetOwned};
    pub(super) mod item;
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub(super) mod top;
    #[doc(inline)]
    pub use {
        item::Item,
        leaf::Leaf,
        node::Node,
        tier::{Nested, Tier},
        top::Top,
    };
}

pub(crate) use interface::UncheckedSetHash;
