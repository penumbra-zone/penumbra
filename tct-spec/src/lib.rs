#![allow(clippy::type_complexity)]

pub mod block;
pub mod index;
pub mod tree;
pub use tree::{Leaf, Node, Tier, Tree};

mod error;
pub use error::InsertError;

use penumbra_tct::{
    internal::{
        active::Insert,
        hash::{CachedHash, GetHash, Hash},
    },
    Commitment,
};

/// The maximum capacity for any tier of the tree: 4^8 = 65,536.
pub const TIER_CAPACITY: usize = 4usize.pow(8);
