//! The internal implementation of the tree, exposed here for documentation.
//!
//! This module and its submodules should not be expected to follow semantic versioning.

pub mod hash;
pub mod height;
pub mod index;
pub mod item;
pub mod path;
pub mod proof;
pub mod three;

mod insert;
mod interface;
pub use interface::Witness;

pub mod active {
    //! [`Active`] things can be inserted into and updated, always representing the rightmost (most
    //! recently inserted) element of a tree.
    //!
    //! The structure of a single [`Tier`] contains eight [`Node`]s, the bottom-most of which
    //! contains a [`Leaf`].
    use super::interface;
    pub use interface::{Active, Focus, Forget, Full};
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub use super::insert::Insert;
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
    pub(super) mod leaf;
    pub(super) mod node;
    pub(super) mod tier;
    pub use leaf::Leaf;
    pub use node::children;
    pub use node::Node;
    pub use tier::{Nested, Tier};
}
