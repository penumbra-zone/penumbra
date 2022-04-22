#![allow(clippy::type_complexity)]

pub mod index;
pub mod tree;

mod error;
pub use error::InsertError;

use penumbra_tct::{
    block, epoch,
    internal::{
        active::Insert,
        hash::{CachedHash, GetHash, Hash},
    },
    Commitment,
};
