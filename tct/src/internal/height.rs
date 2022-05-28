//! Type level machinery used to statically determine the height of a tree, to ensure the correct
//! domain separator is used when hashing at any height.
//!
//! Because the height of a tree is inferred by the type system, this means that bugs where the
//! wrong height is used to compute a subtree's hashing domain separator are greatly reduced.
//!
//! This module contains type-level code for computing the height of structures and translating an
//! unary representation good for type-level constraints ([`Succ`] and [`Zero`]) into constant
//! `u64`s suitable for value-level computation.

use crate::prelude::*;

/// Trait identifying the statically-known height of a given tree element.
///
/// This is used to differentiate the hashes at each level of the tree.
pub trait Height {
    /// The height of this type above the leaves of the tree.
    type Height: Path;
}

/// The constant `u8` associated with each unary height.
pub trait IsHeight: sealed::IsHeight {
    /// The number for this height.
    const HEIGHT: u8;
}

/// Height zero.
pub struct Zero;

impl IsHeight for Zero {
    const HEIGHT: u8 = 0;
}

/// Height `N + 1`.
pub struct Succ<N>(N);

impl<N: IsHeight> IsHeight for Succ<N> {
    const HEIGHT: u8 = if let Some(n) = N::HEIGHT.checked_add(1) {
        n
    } else {
        panic!("height overflow: can't construct something of height > u8::MAX")
    };
}

/// Seal the `IsHeight` trait so that only `Succ` and `Zero` can inhabit it.
mod sealed {
    use super::{Succ, Zero};

    pub trait IsHeight {}
    impl IsHeight for Zero {}
    impl<N: IsHeight> IsHeight for Succ<N> {}
}
