//! The tiered commitment tree for Penumbra.
//!
//! ```ascii,no_run
//! Eternity┃           ╱╲ ◀───────────── Anchor
//!     Tree┃          ╱││╲               = Global Tree Root
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Global Tree Leaf
//!                         ▲             = Epoch Root
//!                      ┌──┘
//!                      │
//!                      │
//!    Epoch┃           ╱╲ ◀───────────── Epoch Root
//!     Tree┃          ╱││╲
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Epoch Leaf
//!                  ▲                    = Block Root
//!                  └───┐
//!                      │
//!                      │
//!    Block┃           ╱╲ ◀───────────── Block Root
//!     Tree┃          ╱││╲
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Block Leaf
//!                                       = Note Commitment
//! ```

// Cargo doc complains if the recursion limit isn't higher, even though cargo build succeeds:
#![recursion_limit = "256"]
#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate async_stream;

mod commitment;
mod index;
mod proof;
mod random;
mod tree;
mod visualize;
mod witness;

pub mod error;
pub mod storage;
pub mod structure;
pub mod validate;

pub use commitment::Commitment;
pub use internal::hash::Forgotten;
pub use proof::Proof;
pub use tree::{Position, Root, Tree};
pub use witness::Witness;

#[cfg(any(doc, feature = "internal"))]
pub mod internal;
#[cfg(not(any(doc, feature = "internal")))]
mod internal;

#[cfg(feature = "live-view")]
pub mod live;

pub mod builder {
    //! Builders for individual epochs and blocks: useful when constructing a [`Tree`](super::Tree)
    //! in parallel, but unnecessary in a single thread.

    pub mod epoch {
        //! Build individual epochs to insert into [`Tree`](super::super::Tree)s.
        pub use crate::tree::epoch::*;
    }

    pub mod block {
        //! Build individual blocks to insert into [`epoch::Builder`](super::epoch::Builder)s or
        //! [`Tree`](super::super::Tree)s.
        pub use crate::tree::epoch::block::*;
    }
}

// A crate-internal prelude to make things easier to import
mod prelude {
    pub(crate) use super::{
        error::proof::VerifyError,
        index,
        internal::{
            complete::{self, Complete, ForgetOwned, OutOfOrderOwned},
            frontier::{
                self, Focus, Forget, Frontier, Full, GetPosition, Insert, InsertMut, Item,
                OutOfOrder,
            },
            hash::{CachedHash, Forgotten, GetHash, Hash, OptionHash},
            height::{Height, IsHeight, Succ, Zero},
            interface::Witness,
            path::{self, AuthPath, Path, WhichWay},
            three::{Elems, ElemsMut, IntoElems, Three},
            UncheckedSetHash,
        },
        storage::{self, Read, Write},
        structure::{self, Kind, Node, Place},
        Commitment, Position, Proof, Root, Tree,
    };
}

#[cfg(feature = "arbitrary")]
/// Generation of random [`Commitment`]s for testing.
pub mod proptest {
    #[doc(inline)]
    pub use super::commitment::FqStrategy;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Tree, [u8; 896]);
    }

    #[test]
    fn check_eternity_proof_size() {
        static_assertions::assert_eq_size!(Proof, [u8; 2344]);
    }
}
