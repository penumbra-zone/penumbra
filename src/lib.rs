//! The tiered commitment tree for Penumbra.

#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

pub mod internal;
mod item;

#[doc(inline)]
pub use item::Item;

#[doc(inline)]
pub use internal::{active::Insert, hash::Hash};

use internal::{
    active::{Active, Focus, Full},
    complete::Complete,
    hash::GetHash,
    height::Height,
};
