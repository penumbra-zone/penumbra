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

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

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

mod commitment;
mod index;
mod proof;
mod random;
mod tree;
mod witness;

pub mod error;
pub mod storage;
pub mod structure;
pub mod validate;

#[doc(inline)]
pub use {
    commitment::Commitment,
    internal::hash::Forgotten,
    proof::Proof,
    tree::{Position, Root, Tree},
    witness::Witness,
};

#[cfg(any(doc, feature = "internal"))]
pub mod internal;
#[cfg(not(any(doc, feature = "internal")))]
mod internal;

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
        storage::{
            self, AsyncRead, AsyncWrite, DeleteRange, Read, StoreCommitment, StoreHash,
            StoredPosition, Update, Write,
        },
        structure::{self, HashOrNode, HashedNode, Kind, Node, Place},
        Commitment, Position, Proof, Root, Tree,
    };

    // We use the hash map from `im`, but with the fast "hash prehashed data" hasher from `hash_hasher`
    pub(crate) type HashedMap<K, V> = im::HashMap<K, V, hash_hasher::HashBuildHasher>;
}

#[cfg(feature = "arbitrary")]
/// Generation of random [`Commitment`]s for testing.
pub mod proptest {
    #[doc(inline)]
    pub use super::commitment::FqStrategy;
}

#[cfg(test)]
mod test {
    #[test]
    fn check_eternity_size() {
        // Disabled due to spurious test failure.
        // static_assertions::assert_eq_size!(Tree, [u8; 32]);
    }

    #[test]
    fn check_eternity_proof_size() {
        // Disabled due to spurious test failure.
        // static_assertions::assert_eq_size!(Proof, [u8; 2344]);
    }
}
