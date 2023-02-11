//! Storage and management of chain state.
//!
//! This crate provides a versioned, verifiable key-value store that also
//! supports lightweight, copy-on-write snapshots and transactional semantics.
//! The [`Storage`] type is a handle for an instance of a backing store,
//! implemented using RocksDB.  The storage records a sequence of versioned
//! [`State`]s.  The [`State`] type is a lightweight snapshot of a particular
//! version of the chain state.
//!
//! Each [`State`] instance can also be used as a copy-on-write fork to build up
//! changes before committing them to persistent storage.  The
//! [`StateTransaction`] type collects a group of writes, which can then be
//! applied to the (in-memory) [`State`] fork.  Finally, the changes accumulated
//! in the [`State`] instance can be committed to the persistent [`Storage`].
//!
//! Reads are performed with the [`StateRead`] trait, implemented by both
//! [`State`] and [`StateTransaction`], and reflect any currently cached writes.
//! Writes are performed with the [`StateWrite`] trait, which is only
//! implemented for [`StateTransaction`].
//!
//! The storage system provides two data stores:
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
//! While the primary key-value store records byte values, it is intended for
//! use with Protobuf-encoded data.  To this end, the [`StateRead`] and
//! [`StateWrite`] traits have provided methods that use the
//! [`penumbra_proto::Protobuf`] trait to automatically (de)serialize into proto
//! or domain types, allowing its use as an object store.

mod metrics;
mod snapshot;
mod snapshot_cache;
mod state;
mod storage;

use snapshot::Snapshot;

pub use crate::metrics::register_metrics;
pub use jmt::{ics23_spec, RootHash};
pub use state::{ArcStateExt, State, StateRead, StateTransaction, StateWrite};
pub use storage::{StateNotification, Storage, TempStorage};

pub mod future;
