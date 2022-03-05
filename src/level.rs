use crate::{Active as _, Complete as _};

/// This is used to make it easier to declare the 8-deep nested type of `Complete`.
macro_rules! complete_type {
    ($node:ident, $leaf:ident :) => {
	$leaf
    };
    ($node:ident, $leaf:ident : @ $($rest:tt)*) => {
	$node<complete_type!($node, $leaf : $($rest)*)>
    };
}

/// This is used to make it easier to declare the 8-deep, 8-wide nested type of `Active`.
macro_rules! active_type {
    ($active:ident, $node:ident, $complete_leaf:ident, $active_leaf:ident :) => {
	$active_leaf
    };
    ($active:ident, $node:ident, $complete_leaf:ident, $active_leaf:ident : @ $($rest:tt)*) => {
	$active<
	    complete_type!($node, $complete_leaf : $($rest)*),
	    active_type!($active, $node, $complete_leaf, $active_leaf : $($rest)*)
	>
    };
}

/// Unit tests for the macros to ensure that they are producing the right shapes of types.
#[cfg(test)]
mod test {
    use static_assertions::assert_type_eq_all as type_eq;

    use crate::{
        node::{Active as A, Complete as N},
        Commitment,
    };
    #[allow(unused)]
    type F = crate::leaf::Active<Commitment, 0>;
    type L = crate::leaf::Complete<Commitment, 0>;

    #[test]
    fn test_complete_type() {
        type_eq!(complete_type!(N, L: @@), N<N<L>>);
    }

    #[test]
    fn test_active_type() {
        type_eq!(active_type!(A, N, L, L: @@), A<N<L>, A<L, L>>);
    }

    #[test]
    fn test_duals() {
        type_eq!(
            <complete_type!(N, L: @@@@@@@@) as crate::Complete>::Active,
            active_type!(A, N, L, F: @@@@@@@@)
        );

        type_eq!(
            <active_type!(A, N, L, F: @@@@@@@@) as crate::Active>::Complete,
            complete_type!(N, L: @@@@@@@@)
        );
    }
}

mod active;
mod complete;
use {active::Active, complete::Complete};

// #[derive(Debug, Clone, PartialEq, Eq)]
// enum Inner<L> {
//     Empty,
//     Active(Active<L>),
//     Complete(Complete<L>),
// }

// impl<L> Default for Inner<L> {
//     fn default() -> Self {
//         Inner::Empty
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct Level<L>(Inner<L>);

// impl<L> Default for Level<L> {
//     fn default() -> Self {
//         Level(Inner::default())
//     }
// }

// impl<L> Level<L> {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn insert(&mut self, value: L) -> Result<(), L> {
//         match self.0 {
//             Inner::Empty => self.0 = Inner::Active(Active::singleton(value)),
//             Inner::Complete(_) => return Err(value),
//             Inner::Active(active) => match active.insert(value) {
//                 Ok(active) => self.0 = Inner::Active(active),
//                 Err((value, complete)) => {
//                     self.0 = Inner::Complete(complete);
//                     return Err(value);
//                 }
//             },
//         }
//         Ok(())
//     }
// }
