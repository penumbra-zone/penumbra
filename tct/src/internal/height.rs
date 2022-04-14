//! All structures in this crate have a statically known height, which is used to compute hashes
//! that vary by height.
//!
//! This module contains type-level machinery for computing the height of structures and translating
//! an unary representation good for type-level constraints ([`Succ`] and [`Zero`]) into constant
//! `u64`s suitable for value-level computation.

/// Trait identifying the statically-known height of a given tree element.
///
/// This is used to differentiate the hashes at each level of the tree.
pub trait Height {
    /// The height of this type above the leaves of the tree.
    type Height: IsHeight;
}

/// The constant `u64` associated with each unary height.
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
    const HEIGHT: u8 = N::HEIGHT + 1;
}

/// Seal the `IsHeight` trait so that only `Succ` and `Zero` can inhabit it.
mod sealed {
    use super::{Succ, Zero};

    pub trait IsHeight {}
    impl IsHeight for Zero {}
    impl<N: IsHeight> IsHeight for Succ<N> {}
}
