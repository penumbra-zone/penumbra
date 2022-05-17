//! The executable reference spec for [`penumbra_tct`], being a *non*-incremental, *non*-sparse
//! commitment tree.
//!
//! ⚠️ Don't use this in production code: it is slower, less flexible, and less type-safe.
//!
//! This specification implements an almost-identical interface to the tiered commitment tree in the
//! main crate. However, unlike the main crate, it separates the *construction* of trees from their
//! *examination*. The types [`eternity::Builder`], [`epoch::Builder`], and [`block::Builder`] are
//! used to build trees; they support all the mutable operations of [`Eternity`](crate::Eternity),
//! [`Epoch`](crate::Epoch), and [`Block`](crate::Block), but none of their immutable observations.
//! Each builder can be built into a [`spec::Eternity`](crate::spec::Eternity),
//! [`spec::Epoch`](crate::spec::Epoch), or [`spec::Block`](crate::spec::Block), respectively, which
//! each support all the immutable methods of their main-crate counterparts.

#![allow(clippy::type_complexity)]
#![warn(missing_docs)]

pub mod block;
pub mod index;
pub mod tree;
pub use tree::{Internal, Leaf, Tier, Tree};

mod error;
pub use error::InsertError;

use penumbra_tct::{
    self as tct,
    internal::{
        frontier::Insert,
        hash::{CachedHash, GetHash, Hash},
        path::WhichWay,
    },
    Commitment, Witness,
};

use std::collections::VecDeque as List;

/// The maximum capacity for any tier of the tree: 4^8 = 65,536.
pub const TIER_CAPACITY: usize = 4usize.pow(8);
