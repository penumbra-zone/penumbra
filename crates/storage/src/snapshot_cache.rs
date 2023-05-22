use crate::Snapshot;
use std::{cmp, collections::VecDeque};

use anyhow::anyhow;

/// A circular cache for storing [`Snapshot`]s.
///
/// # Usage
///
/// [`Snapshot`]s are inserted in the cache using the [`push`] or [`try_push`]
/// methods. If the cache is full, the oldest entry will be evicted to make space
/// for the newer entry.
///
/// # Constraints
///
/// [`Snapshot`]s must be inserted sequentially relative to their [`jmt::Version`]
/// numbers, and have consecutive version numbers.
pub struct SnapshotCache {
    /// A sequence of increasingly recent [`Snapshot`]s.
    cache: VecDeque<Snapshot>,
    /// The max length and capacity of the cache.
    max_size: usize,
}

impl SnapshotCache {
    /// Creates a [`SnapshotCache`] with `max_size` capacity, and inserts an initial `Snapshot` in
    /// it. If the specified capacity is zero, the cache will default to having size 1.
    pub fn new(initial: Snapshot, max_size: usize) -> Self {
        let max_size = cmp::max(max_size, 1);
        let mut cache = VecDeque::with_capacity(max_size);
        cache.push_front(initial);

        Self { cache, max_size }
    }

    /// Attempts to insert a [`Snapshot`] entry into the cache. If the cache is full, the oldest
    /// entry will be evicted to make space.
    ///
    /// [`Snapshot`]s must be inserted sequentially relative to their `jmt::Version`s and have
    /// consecutive version numbers.
    ///
    /// ## Errors
    ///
    /// The method will return an error if the supplied `snapshot` has a version number that is:
    ///
    /// - stale i.e. older than the latest snapshot
    ///
    /// - skipping a version i.e. the difference between their version numbers is greater than 1
    pub fn try_push(&mut self, snapshot: Snapshot) -> anyhow::Result<()> {
        let latest_version = self.latest().version();
        if latest_version.wrapping_add(1) != snapshot.version() {
            return Err(anyhow!("snapshot_cache: trying to insert stale snapshots."));
        }

        if self.cache.len() >= self.max_size {
            self.cache.pop_back();
        }

        self.cache.push_front(snapshot);
        Ok(())
    }

    /// Returns the latest inserted `Snapshot`.
    pub fn latest(&self) -> Snapshot {
        self.cache
            .front()
            .map(Clone::clone)
            .expect("snapshot_cache cannot be empty")
    }

    /// Attempts to fetch a [`Snapshot`] with a matching `jmt::Version`, and returns `None` if none
    /// was found.
    pub fn get(&self, version: jmt::Version) -> Option<Snapshot> {
        let latest_version = self.latest().version();
        // We compute the offset assuming that snapshot entries are cached
        // such that the delta between entries is always 1.
        let offset = latest_version.wrapping_sub(version) as usize;
        self.cache
            .get(offset)
            .map(Clone::clone)
            .filter(|s| s.version() == version)
    }
}

#[cfg(test)]
mod test {
    use crate::snapshot::Snapshot;
    use crate::snapshot_cache::SnapshotCache;
    use crate::storage::Storage;

    async fn create_storage_instance() -> Storage {
        use tempfile::tempdir;
        // create a storage backend for testing
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("snapshot-cache-testing.db");

        Storage::load(file_path).await.unwrap()
    }

    #[tokio::test]
    /// `SnapshotCache` constructed with zero capacity instead defaults to one.
    async fn fail_zero_capacity() {
        let storage = create_storage_instance().await;
        let db = storage.db();
        let snapshot = storage.latest_snapshot();
        let mut cache = SnapshotCache::new(snapshot, 0);

        // Check that the cache has a capacity at least 1
        assert!(cache.get(u64::MAX).is_some());
        let new_snapshot = Snapshot::new(db, 0);
        cache.try_push(new_snapshot).unwrap();

        // Check that the cache has a capacity of exactly 1
        assert!(cache.get(u64::MAX).is_none());
        assert!(cache.get(0).is_some());
    }

    #[tokio::test]
    /// Fails to insert snapshot entries that are older than the latest'
    async fn fail_insert_stale_snapshot() {
        let storage = create_storage_instance().await;
        let db_handle = storage.db();
        let snapshot = storage.latest_snapshot();
        let mut cache = SnapshotCache::new(snapshot, 1);
        let stale_snapshot = Snapshot::new(db_handle, 1);
        cache
            .try_push(stale_snapshot)
            .expect_err("should fail to insert a stale entry in the snapshot cache");
    }

    #[tokio::test]
    /// Fails to insert snapshot entries that have a version gap.
    async fn fail_insert_gapped_snapshot() {
        let storage = create_storage_instance().await;
        let db_handle = storage.db();
        let snapshot = Snapshot::new(db_handle.clone(), 0);
        let mut cache = SnapshotCache::new(snapshot, 2);
        let snapshot = Snapshot::new(db_handle, 2);
        cache
            .try_push(snapshot)
            .expect_err("should fail to insert snapshot with skipped version number");
    }

    #[tokio::test]
    /// Checks that we handle pre-genesis `jmt::Version` correctly.
    async fn cache_manage_pre_genesis() {
        let storage = create_storage_instance().await;
        let db_handle = storage.db();
        let snapshot = storage.latest_snapshot();

        // Create a cache of size 10, populated with one entry with version: u64::MAX
        let mut cache = SnapshotCache::new(snapshot, 10);

        // Fill the entire cache by inserting 9 more entries.
        for i in 0..9 {
            let snapshot = Snapshot::new(db_handle.clone(), i);
            cache.try_push(snapshot).unwrap();
        }

        // The cache is full, check that the oldest entry is still in the cache.
        assert!(cache.get(u64::MAX).is_some());

        // Push another snapshot in the cache, this should cause eviction of the oldest entry
        // alone.
        let new_snapshot = Snapshot::new(db_handle, 9);
        cache.try_push(new_snapshot).unwrap();

        // Check that the pre-genesis entry has been evicted!
        assert!(cache.get(u64::MAX).is_none());

        // Check that all the other entries are still in the cache.
        for i in 0..10 {
            assert!(cache.get(i).is_some());
        }
    }

    #[tokio::test]
    /// Checks that inserting on a full cache exclusively evicts the oldest snapshots.
    async fn drop_oldest_snapshot() {
        let storage = create_storage_instance().await;
        let db_handle = storage.db();
        let snapshot = Snapshot::new(db_handle.clone(), 0);

        // Create a cache of size 10, populated with a snapshot at version 0.
        let mut cache = SnapshotCache::new(snapshot, 10);

        // Saturate the cache by inserting 9 more entries.
        for i in 1..10 {
            let snapshot = Snapshot::new(db_handle.clone(), i);
            cache.try_push(snapshot).unwrap()
        }

        // Check that the oldest value is still present:
        assert!(cache.get(0).is_some());

        // Insert a new value that should overflow the cache.
        let snapshot = Snapshot::new(db_handle, 10);
        cache.try_push(snapshot).unwrap();

        // Check that the oldest value has been dropped.
        assert!(cache.get(0).is_none());

        // Check that the front of the cache is the latest inserted snapshot.
        assert_eq!(cache.latest().version(), 10);

        // Check that all the other snapshots are still present in the cache.
        for i in 1..11 {
            assert!(cache.get(i).is_some());
        }
    }
}
