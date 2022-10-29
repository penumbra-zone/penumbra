use std::ops::Deref;
use std::sync::Arc;

/// A wrapper type that acts as a `rocksdb::Snapshot` of an `Arc`'d database
/// handle.
///
/// This works around a limitation of the `rocksdb` API: the `rocksdb::Snapshot`
/// can only take a borrowed database handle, not an `Arc`'d one, so the
/// lifetime of the `rocksdb::Snapshot` is bound to the lifetime of the borrowed
/// handle.  Instead, this wrapper type bundles an `Arc`'d handle together with
/// the `rocksdb::Snapshot`, so that the database is guaranteed to live at least
/// as long as any snapshot of it.
pub struct RocksDbSnapshot {
    /// The snapshot itself.  It's not really `'static`, so it's on us to ensure
    /// that the database stays live as long as the snapshot does.
    inner: rocksdb::Snapshot<'static>,
    /// The raw pointer form of the Arc<DB> we use to guarantee the database
    /// lives at least as long as the snapshot.  We create this from the Arc<DB>
    /// in the constructor, pass it to the snapshot on creation, and then
    /// convert it back into an Arc in the drop impl to decrement the refcount.
    ///
    /// Arc::into_raw consumes the Arc instance but does not decrement the
    /// refcount.  This means that we cannot accidentally drop the Arc while
    /// using the raw pointer.  Instead, we must explicitly convert the raw
    /// pointer back into an Arc when we're finished using it, and only then
    /// drop it.
    raw_db: *const rocksdb::DB,
}

// Safety requires that the inner snapshot instance must never live longer than
// the wrapper.  We're assured that this is the case, because we only return a
// borrow of the inner snapshot, and because `rocksdb::Snapshot` is neither
// `Copy` nor `Clone`.
//
// We're also reasonably certain that the upstream crate will not add such an
// implementation in the future, because its drop impl is used to make the FFI
// call that discards the in-memory snapshot, so it would not be safe to add
// such an implementation.
impl Deref for RocksDbSnapshot {
    type Target = rocksdb::Snapshot<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RocksDbSnapshot {
    /// Creates a new snapshot of the given `db`.
    pub fn new(db: Arc<rocksdb::DB>) -> Self {
        // First, convert the Arc<DB> into a raw pointer.
        let raw_db = Arc::into_raw(db);
        // Next, use the raw pointer to construct a &DB instance with a fake
        // 'static lifetime, and use that instance to construct the inner
        // Snapshot.
        let static_db: &'static rocksdb::DB = unsafe { &*raw_db };
        let inner = rocksdb::Snapshot::new(static_db);

        Self { inner, raw_db }
    }
}

impl Drop for RocksDbSnapshot {
    fn drop(&mut self) {
        // Now that we know we're finished with the `Snapshot`, we can
        // reconstruct the `Arc` and drop it, to decrement the DB refcount.
        unsafe {
            let db = Arc::from_raw(self.raw_db);
            std::mem::drop(db);
        }
    }
}

/// The `Send` implementation is safe because the `rocksdb::Snapshot` is `Send`.
unsafe impl Send for RocksDbSnapshot {}
/// The `Sync` implementation is safe because the `rocksdb::Snapshot` is `Sync`.
unsafe impl Sync for RocksDbSnapshot {}
