//! The executable reference spec: a *non*-incremental, *non*-sparse commitment tree. **Don't use this
//! in production code: it is slower, less flexible, and less type-safe.**
//!
//! This specification implements an almost-identical interface to the tiered commitment tree in the
//! main crate. However, unlike the main crate, it separates the *construction* of trees from their
//! *examination*. The types [`eternity::Builder`], [`epoch::Builder`], and [`block::Builder`] are
//! used to build trees; they support all the mutable operations of [`Eternity`](crate::Eternity),
//! [`Epoch`](crate::Epoch), and [`Block`](crate::Block), but none of their immutable observations.
//! Each builder can be built into a [`spec::Eternity`](crate::spec::Eternity),
//! [`spec::Epoch`](crate::spec::Epoch), or [`spec::Block`](crate::spec::Block), respectively, which
//! each support all the immutable methods of their main-crate counterparts.

use std::collections::VecDeque;

pub mod block;
pub use block::Block;
pub mod epoch;
pub use epoch::Epoch;
pub mod eternity;
pub use eternity::Eternity;
mod error;
mod tree;
pub use error::InsertError;

/// The maximum capacity for any tier of the tree: 4^8 = 65,536.
pub const TIER_CAPACITY: usize = 4usize.pow(8);

/// A single tier of a builder, being a sequence of insertions of the element of the tier (whether
/// that's an individual commitment or a sub-tier).
type Tier<T> = VecDeque<crate::Insert<T>>;
