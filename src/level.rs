/// From input like `N N N L` generate `N<N<N<L>>>`.
///
/// This is used to make it easier to declare the 8-deep nested type of `Complete`.
macro_rules! complete_type {
    ($leaf:ident) => {
	$leaf
    };
    ($node:ident $($rest:tt)+) => {
	$node<complete_type!($($rest)+)>
    };
}

/// From input like `A: N N N L` generate `A<N<N<N<L>>>, A<N<N<L>>, A<N<L>, L>>>`.
///
/// This is used to make it easier to declare the 8-deep, 8-wide nested type of `Active`.
macro_rules! active_type {
    ($segment:ident : $leaf:ident) => {
	$leaf
    };
    ($segment:ident : $node:ident $($rest:tt)+) => {
	$segment<complete_type!($node $($rest)+), active_type!($segment : $($rest)+)>
    };
}

mod active;
mod complete;
pub use {active::Active, complete::Complete};
