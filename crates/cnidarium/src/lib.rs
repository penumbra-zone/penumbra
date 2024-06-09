//! Storage and management of chain state, backed by Jellyfish Merkle Trees and RocksDB.
//!
//! This crate provides a versioned, verifiable key-value store that also
//! supports lightweight, copy-on-write snapshots and transactional semantics.
//! The [`Storage`] type is a handle for an instance of a backing store,
//! implemented using RocksDB.  The storage records a sequence of versioned
//! [`Snapshot`]s.  The [`Snapshot`] type is a lightweight snapshot of a particular
//! version of the chain state.
//!
//! Each [`Snapshot`] instance can also be used as the basis for a copy-on-write
//! fork to build up changes before committing them to persistent storage.  The
//! [`StateDelta`] type collects a group of writes, which can then be applied to
//! the (in-memory) [`StateDelta`] overlay.  Finally, the changes accumulated in the
//! [`StateDelta`] instance can be committed to the persistent [`Storage`].
//!
//! Reads are performed with the [`StateRead`] trait, implemented by both
//! [`Snapshot`] and [`StateDelta`], and reflect any currently cached writes.
//! Writes are performed with the [`StateWrite`] trait, which is only
//! implemented for [`StateDelta`].
//!
//! The storage system provides three data stores:
//!
//! * A verifiable key-value store, with UTF-8 keys and byte values, backed by
//! the Jellyfish Merkle Tree.  The JMT is a sparse merkle tree that records
//! hashed keys, so we also record an index of the keys themselves to allow
//! range queries on keys rather than key hashes. This index, however, is not
//! part of the verifiable consensus state.
//!
//! * A secondary, non-verifiable key-value store with byte keys and byte
//! values, backed directly by RocksDB.  This is intended for use building
//! application-specific indexes of the verifiable consensus state.
//!
//! * A tertiary, in-memory object store. This is intended for use implementing
//! accumulators, like lists of data to be batch-processed at the end of the
//! block.  The object store clones on read to prevent violations of
//! transactional semantics, so it should be used with immutable data structures
//! like those in the `im` crate that implement copy-on-write behavior
//! internally.
//!
//! The storage system also supports prefixed "substores", somewhat similar to
//! the Cosmos SDK's multistore design. Each substore has a separate JMT, whose
//! root hash is written into the base store under the prefix.  This allows use
//! cases like storing IBC data in a subtree.  The substore's non-verifiable
//! store is also stored in a separate RocksDB column family, allowing storage
//! optimizations.
//!
//! Remember that the chain state is a public API.  Mapping from raw byte values
//! to typed data should be accomplished by means of extension traits.  For
//! instance, the `penumbra_proto` crate provides an extension trait to
//! automatically (de)serialize into proto or domain types, allowing its use as
//! an object store.
//!
//! With the `rpc` feature enabled, this crate also provides a GRPC interface to
//! the key-value store using Tonic.
#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// We use `HashMap`s opportunistically.
#![allow(clippy::disallowed_types)]

mod cache;
mod delta;
mod escaped_byte_slice;
mod metrics;
mod read;
mod snapshot;
mod snapshot_cache;
mod storage;
mod store;
#[cfg(test)]
mod tests;
mod utils;
mod write;
mod write_batch;

#[cfg(feature = "metrics")]
pub use crate::metrics::register_metrics;
pub use cache::Cache;
pub use delta::{ArcStateDeltaExt, StateDelta};
pub use escaped_byte_slice::EscapedByteSlice;
pub use jmt::{ics23_spec, RootHash};
pub use read::StateRead;
pub use snapshot::Snapshot;
pub use storage::{Storage, TempStorage};
pub use write::StateWrite;
pub use write_batch::StagedWriteBatch;

pub mod future;

#[cfg(feature = "rpc")]
pub mod rpc;
