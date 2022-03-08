//! The tiered commitment tree for Penumbra.

#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

mod hash;
#[doc(inline)]
pub use hash::{GetHash, Hash};

mod item;
#[doc(inline)]
pub use item::Item;

#[doc(inline)]
pub use internal::height::Height;

pub mod internal;
#[doc(inline)]
pub use internal::active::Insert;
use internal::{
    active::{Active, Focus, Full},
    complete::Complete,
};
