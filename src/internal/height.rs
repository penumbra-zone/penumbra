/// Trait identifying the statically-known height of a given tree element.
///
/// This is used to differentiate the hashes at each level of the tree.
pub trait Height {
    /// The height of this type above the leaves of the tree.
    type Height: IsHeight;
}

/// The constant `usize` associated with each unary height.
pub trait IsHeight: sealed::IsHeight {
    /// The number for this height.
    const HEIGHT: usize;
}

/// Height zero.
pub struct Zero;

impl IsHeight for Zero {
    const HEIGHT: usize = 0;
}

/// Height `N + 1`.
pub struct Succ<N>(N);

impl<N: IsHeight> IsHeight for Succ<N> {
    const HEIGHT: usize = N::HEIGHT + 1;
}

/// Seal the `IsHeight` trait so that only `Succ` and `Zero` can inhabit it.
mod sealed {
    use super::{Succ, Zero};

    pub trait IsHeight {}
    impl IsHeight for Zero {}
    impl<N: IsHeight> IsHeight for Succ<N> {}
}
