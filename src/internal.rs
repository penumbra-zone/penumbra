//! The internal implementation of the tree, exposed here for documentation.
//!
//! This module and its submodules should not be expected to follow semantic versioning.

pub mod three;

pub mod height;

mod interface;

mod leaf {
    mod active;
    mod complete;
    #[doc(inline)]
    pub use {active::Active, complete::Complete};
}

mod node {
    mod active;
    mod complete;
    #[doc(inline)]
    pub use {active::Active, complete::Complete};
}

mod tier {
    mod active;
    mod complete;
    #[doc(inline)]
    pub use {active::Active, complete::Complete};
}

pub mod active {
    //! [`Active`] things can be inserted into and updated, always representing the rightmost (most
    //! recently inserted) element of a tree.
    //!
    //! The structure of a single [`Tier`] contains eight [`Node`]s, the bottom-most of which
    //! contains a [`Leaf`].
    use super::{interface, leaf, node, tier};
    pub use interface::{Active, Focus, Full, Insert};
    pub use leaf::Active as Leaf;
    pub use node::Active as Node;
    pub use tier::Active as Tier;
}

pub mod complete {
    //! [`Complete`] things are sparse representations of only the data that was inserted using
    //! [`Insert::Keep`](crate::Insert::Keep), with the data that was inserted using
    //! [`Insert::Hash`](crate::Insert::Hash) being pruned eagerly.
    //!
    //! The structure of a single [`Tier`] contains eight levels of [`Node`]s, the bottom-most level
    //! of which contains [`Leaf`]s.
    use super::{interface, leaf, node, tier};
    pub use interface::Complete;
    pub use leaf::Complete as Leaf;
    pub use node::Complete as Node;
    pub use tier::Complete as Tier;
}
