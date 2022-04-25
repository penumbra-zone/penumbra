//! The internal implementation of the tree, exposed here for documentation.
//!
//! This module and its submodules should not be expected to follow semantic versioning.
//!
//! ## Structure of Implementation
//!
//! The tiered commitment tree is not accessed directly _as a tree_; rather, the [`Eternity`],
//! [`Epoch`], and [`Block`] structs from the top level of the crate contain a tree together with a
//! hashmap which maps commitments to their corresponding index within the tree. This `internal`
//! module and all its submodules concern themselves solely with the implementation of the tree
//! itself, wherein commitments and their authentication paths are accessed by index. The
//! surrounding pieces of the crate make use of the internal-facing API exposed by this module to
//! implement an external API specific to the three-tiered eternity/epoch/block commitment tree
//! required by Penumbra.
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
//! inductive non-generic enumeration types with recursive methods). It's worth keeping this in mind
//! while reading the code, because it will help to clarify the interplay of various traits.
//!
//! ## Tiers of Nodes of Leaves of Items: Frontier and Complete
//!
//! The primary exports of this module is the type [`frontier::Tier`]. This type is a quadtree where
//! every internal node is annotated with the hash of its children, into which leaves (all at depth
//! 8) are inserted in left to right order. The "frontier" represents the path from the root to the
//! rightmost leaf, at every level containing any leftward siblings of a given frontier node, each
//! of which is a [`complete`] tree which stores the finalized bulk of the items inserted into the
//! tree. As new leaves are created, the frontier zig-zags rightwards, pushing finalized portions of
//! itself into the leftward complete tree.
//!
//! As described above, a variety of recursively defined traits are used to define the behavior of
//! trees. The [`Frontier`] trait defines the operations possible on a frontier of a tree, while the
//! [`Focus`] trait defines how to operate over the tip of a frontier, and the [`Forget`] trait
//! defines how to remove witnessed leaves from a frontier.
//!
//! While the frontier changes on every inserted leaf, within the complete portion of the tree the
//! only changes that occur are when leaves are forgotten and their containing nodes pruned. As a
//! result, the traits exposed by the [`complete`] module are merely [`Complete`], which is a marker
//! trait used to ensure that every [`Frontier`] type is paired with a unique corresponding type of
//! complete tree, and the [`ForgetOwned`] trait, which defines an equivalent to
//! [`frontier::Forget`] that is applicable to the by-value usage pattern of complete trees.
//!
//! ## Utilities Used Across The Implementation
//!
//! The [`hash`] module defines the core [`Hash`](hash::Hash) type, which is used internally to
//! represent hashes, as well as the [`GetHash`] trait, which is defined on most structures within
//! this crate and describes how to compute their hash (caching the result if required). It also
//! defines a [`CachedHash`] type, which is used for lazy evaluation of the hashes of internal
//! nodes.
//!
//! The [`height`] module defines the [`Height`] trait and several associated pieces of type-level
//! machinery, used to statically determine the height of a tree. Because the height of a tree is
//! inferred by the type system, this means that bugs where the wrong height is used to compute a
//! subtree's hashing domain separator are greatly reduced.
//!
//! The [`path`] module defines the type of authentication paths into the tree, generically for
//! trees of any height. These are wrapped in more specific domain types by the exposed crate API to
//! make it more comprehensible.
//!
//! The [`proof`] module defines (transparent) merkle inclusion proofs generically for trees of any
//! height. These are wrapped in mode specific domain types by the exposed crate API to make it more
//! comprehensible.
//!
//! The [`three`] module defines a wrapper around [`Vec`] for vectors whose length is at most 3
//! elements. This is used in the implementation of [`frontier::Node`]s to store the lefthand siblings of the
//! frontier's rightmost child, which must number at most 3 (because nodes must have at most 4
//! children total).

pub mod hash;
pub mod height;
pub mod path;
pub mod proof;
pub mod three;

mod insert;
pub mod interface;

pub mod frontier {
    //! [`Frontier`] things can be inserted into and updated, always representing the rightmost (most
    //! recently inserted) element of a tree.
    //!
    //! The structure of a single [`Tier`] contains eight [`Node`]s, the bottom-most of which
    //! contains a [`Leaf`].
    use super::interface;
    pub use interface::{Focus, Forget, Frontier, Full};
    pub(super) mod item;
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub use super::insert::Insert;
    pub use item::Item;
    pub use leaf::Leaf;
    pub use node::Node;
    pub use tier::{Nested, Tier};
}

pub mod complete {
    //! [`Complete`] things are sparse representations of only the data that was inserted using
    //! [`Insert::Keep`](crate::Insert::Keep), with the data that was inserted using
    //! [`Insert::Hash`](crate::Insert::Hash) being pruned eagerly.
    //!
    //! The structure of a single [`Tier`] contains eight levels of [`Node`]s, the bottom-most level
    //! of which contains [`Leaf`]s.
    use super::interface;
    pub use interface::{Complete, ForgetOwned};
    pub(super) mod item;
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub use item::Item;
    pub use leaf::Leaf;
    pub use node::children;
    pub use node::Node;
    pub use tier::{Nested, Tier};
}
