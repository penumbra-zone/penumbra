/// Trait identifying the statically-known height of a given tree element.
///
/// This is used to differentiate the hashes at each level of the tree.
pub trait Height {
    /// The height of this type above the leaves of the tree.
    type Height: IsHeight;
}

pub trait IsHeight {
    const HEIGHT: usize;
}

pub struct Z;

impl IsHeight for Z {
    const HEIGHT: usize = 0;
}

pub struct S<N>(N);

impl<N: IsHeight> IsHeight for S<N> {
    const HEIGHT: usize = N::HEIGHT + 1;
}
