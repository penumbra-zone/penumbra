use anyhow::Result;
use cnidarium::{StateDelta, StateWrite, Storage};
use tempfile;
use tokio;

#[tokio::test]
/// A simple test that checks that we cannot commit a stale batch to storage.
/// Strategy:
/// Create three state deltas, one that writes to every substore, and two others
/// that target specific substores or none at all.
pub async fn test_write_batch_stale_version_substores() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![
        "ibc".to_string(),
        "dex".to_string(),
        "misc".to_string(),
        "cometbft-data".to_string(),
    ];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;
    let initial_snapshot = storage.latest_snapshot();
    let initial_version = initial_snapshot.version();
    let initial_root_hash = initial_snapshot.root_hash().await?;
    assert_eq!(
        initial_version,
        u64::MAX,
        "initial version should be u64::MAX"
    );
    assert_eq!(initial_root_hash.0, [0u8; 32]);

    /* ************************ Prepare three deltas ************************** */
    // Our goal is to check that we can't commit a batch with a stale version.
    // We create three deltas:
    // 1. Empty delta
    // 2. Delta that writes to one substore
    // 3. Delta that writes to each substore and also writes to the main store

    /* We create an empty delta that writes no keys. */
    let delta_1 = StateDelta::new(initial_snapshot);
    let write_batch_1 = storage.prepare_commit(delta_1).await?;
    let version_1 = write_batch_1.version();
    let root_hash_1 = write_batch_1.root_hash().clone();
    assert_eq!(version_1, initial_version.wrapping_add(1));
    assert_ne!(root_hash_1.0, initial_root_hash.0);

    // We check that merely preparing a batch does not write anything.
    let state_snapshot = storage.latest_snapshot();
    assert_eq!(state_snapshot.version(), initial_version);
    assert_eq!(state_snapshot.root_hash().await?.0, initial_root_hash.0);
    for prefix in substore_prefixes.iter() {
        // We didn't write to any substores, so their version should be unchanged.
        assert_eq!(
            write_batch_1
                .substore_version(prefix)
                .expect("substore exists"),
            u64::MAX
        )
    }

    /* We create a new delta that writes to a single substore. */
    let mut delta_2 = StateDelta::new(state_snapshot.clone());
    delta_2.put_raw("ibc/key".to_string(), [1u8; 32].to_vec());
    let write_batch_2 = storage.prepare_commit(delta_2).await?;
    let version_2 = write_batch_2.version();
    let root_hash_2 = write_batch_2.root_hash();
    assert_eq!(version_2, initial_version.wrapping_add(1));
    assert_ne!(root_hash_2.0, initial_root_hash.0);

    // Now, we check that the version for the main store is incremented, and
    // only the version for the ibc substore is incremented.
    assert_eq!(write_batch_2.version(), initial_version.wrapping_add(1));
    assert_eq!(
        write_batch_2
            .substore_version("ibc")
            .expect("substore_exists"),
        initial_version.wrapping_add(1)
    );
    for prefix in substore_prefixes.iter().filter(|p| *p != "ibc") {
        assert_eq!(
            write_batch_2
                .substore_version(prefix)
                .expect("substore exists"),
            u64::MAX
        )
    }

    /* We create a new delta that writes to each substore. */
    let mut delta_3 = StateDelta::new(state_snapshot);
    for substore_prefix in substore_prefixes.iter() {
        let key = format!("{}/key", substore_prefix);
        tracing::debug!(?key, "adding to delta_1");
        delta_3.put_raw(key, [1u8; 32].to_vec());
    }
    let write_batch_3 = storage.prepare_commit(delta_3).await?;
    let version_3 = write_batch_3.version();
    let root_hash_3 = write_batch_3.root_hash().clone();

    // Once again, we check that we incremented the main store version.
    assert_eq!(version_3, initial_version.wrapping_add(1));
    assert_ne!(root_hash_3.0, initial_root_hash.0);
    // In addition to that, we check that we incremented the version of each substore.
    for prefix in substore_prefixes.iter() {
        assert_eq!(
            write_batch_3
                .substore_version(prefix)
                .expect("substore exists"),
            initial_version.wrapping_add(1)
        )
    }

    /* Persist `write_batch_1` and check that the two other (stale) deltas cannot be applied. */
    let final_root = storage
        .commit_batch(write_batch_1)
        .expect("committing batch 3 should work");
    let final_snapshot = storage.latest_snapshot();
    assert_eq!(root_hash_1.0, final_root.0);
    assert_eq!(root_hash_1.0, final_snapshot.root_hash().await?.0);
    assert_eq!(version_1, final_snapshot.version());
    assert!(
        storage.commit_batch(write_batch_2).is_err(),
        "committing batch 2 should fail"
    );
    assert!(
        storage.commit_batch(write_batch_3).is_err(),
        "committing batch 3 should fail"
    );

    Ok(())
}

#[tokio::test]
/// Test that we can commit a batch without incrementing the substore versions if there are no
/// keys to write.
pub async fn test_two_empty_writes() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![
        "ibc".to_string(),
        "dex".to_string(),
        "misc".to_string(),
        "cometbft-data".to_string(),
    ];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;
    let initial_snapshot = storage.latest_snapshot();
    let initial_version = initial_snapshot.version();
    let initial_root_hash = initial_snapshot.root_hash().await?;
    assert_eq!(
        initial_version,
        u64::MAX,
        "initial version should be u64::MAX"
    );
    assert_eq!(initial_root_hash.0, [0u8; 32]);

    let mut delta_1 = StateDelta::new(initial_snapshot);
    for substore_prefix in substore_prefixes.iter() {
        let key = format!("{}/key", substore_prefix);
        tracing::debug!(?key, "adding to delta_1");
        delta_1.put_raw(key, [1u8; 12].to_vec());
    }
    let write_batch_1 = storage.prepare_commit(delta_1).await?;
    let version_1 = write_batch_1.version();
    let root_hash_1 = write_batch_1.root_hash().clone();

    assert_eq!(version_1, initial_version.wrapping_add(1));
    assert_ne!(root_hash_1.0, initial_root_hash.0);
    for prefix in substore_prefixes.iter() {
        assert_eq!(
            write_batch_1
                .substore_version(prefix)
                .expect("substore exists"),
            initial_version.wrapping_add(1)
        )
    }

    // We check that merely preparing a batch does not write anything.
    let state_snapshot = storage.latest_snapshot();
    assert_eq!(state_snapshot.version(), initial_version);
    assert_eq!(state_snapshot.root_hash().await?.0, initial_root_hash.0);

    /* We create a new delta that writes no keys */
    let delta_2 = StateDelta::new(state_snapshot.clone());
    let write_batch_2 = storage.prepare_commit(delta_2).await?;
    let version_2 = write_batch_2.version();
    let root_hash_2 = write_batch_2.root_hash();
    assert_eq!(version_2, initial_version.wrapping_add(1));
    assert_ne!(root_hash_2.0, initial_root_hash.0);
    assert_eq!(write_batch_2.version(), initial_version.wrapping_add(1));
    for prefix in substore_prefixes.iter() {
        assert_eq!(
            write_batch_2
                .substore_version(prefix)
                .expect("substore exists"),
            initial_version
        )
    }

    let block_1_root = storage
        .commit_batch(write_batch_1)
        .expect("committing batch 3 should work");
    let block_1_snapshot = storage.latest_snapshot();
    let block_1_version = block_1_snapshot.version();
    assert_eq!(root_hash_1.0, block_1_root.0);
    assert_eq!(root_hash_1.0, block_1_snapshot.root_hash().await?.0);
    assert_eq!(version_1, block_1_version);
    assert!(
        storage.commit_batch(write_batch_2).is_err(),
        "committing batch 2 should fail"
    );

    /* We create an empty delta that writes no keys. */
    let delta_3 = StateDelta::new(block_1_snapshot);
    let write_batch_3 = storage.prepare_commit(delta_3).await?;
    let version_3 = write_batch_3.version();
    let root_hash_3 = write_batch_3.root_hash().clone();
    assert_eq!(version_3, block_1_version.wrapping_add(1));

    /* Check that we can apply `write_batch_3` */
    let block_2_root = storage
        .commit_batch(write_batch_3)
        .expect("committing batch 3 should work");
    let block_2_snapshot = storage.latest_snapshot();
    let block_2_version = block_2_snapshot.version();
    assert_eq!(root_hash_3.0, block_2_root.0);
    assert_eq!(root_hash_3.0, block_2_snapshot.root_hash().await?.0);
    assert_eq!(version_3, block_2_version);
    Ok(())
}

#[tokio::test]
/// Test that we can write prepare-commit batches that write to every
/// substore.
/// Intuition: we want to make sure that the version check that guards us from
/// writing stale batches, is working as expected.
pub async fn test_batch_substore() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![
        "ibc".to_string(),
        "dex".to_string(),
        "misc".to_string(),
        "cometbft-data".to_string(),
    ];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;
    let initial_snapshot = storage.latest_snapshot();
    let initial_version = initial_snapshot.version();
    let initial_root_hash = initial_snapshot.root_hash().await?;
    assert_eq!(
        initial_version,
        u64::MAX,
        "initial version should be u64::MAX"
    );
    assert_eq!(initial_root_hash.0, [0u8; 32]);

    for i in 0..100 {
        let snapshot = storage.latest_snapshot();
        let prev_version = snapshot.version();
        let prev_root = snapshot
            .root_hash()
            .await
            .expect("a root hash is available");

        let mut delta = StateDelta::new(snapshot);
        for substore_prefix in substore_prefixes.iter() {
            let key = format!("{}/key_{i}", substore_prefix);
            tracing::debug!(?key, index = i, "adding to delta");
            delta.put_raw(key, [1u8; 12].to_vec());
        }
        let write_batch = storage.prepare_commit(delta).await?;
        let next_version = write_batch.version();
        let next_root = write_batch.root_hash().clone();

        assert_eq!(next_version, prev_version.wrapping_add(1));
        assert_ne!(next_root.0, prev_root.0);
        for prefix in substore_prefixes.iter() {
            assert_eq!(
                write_batch
                    .substore_version(prefix)
                    .expect("substore exists"),
                prev_version.wrapping_add(1)
            )
        }

        // We check that merely preparing a batch does not write anything.
        let state_snapshot = storage.latest_snapshot();
        assert_eq!(state_snapshot.version(), prev_version);
        assert_eq!(state_snapshot.root_hash().await?.0, prev_root.0);

        let block_root = storage
            .commit_batch(write_batch)
            .expect("committing batch 3 should work");
        let block_snapshot = storage.latest_snapshot();
        let block_version = block_snapshot.version();
        assert_eq!(next_root.0, block_root.0);
        assert_eq!(next_root.0, block_snapshot.root_hash().await?.0);
        assert_eq!(next_version, block_version);
    }

    Ok(())
}
