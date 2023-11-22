#![cfg(feature = "migration")]
use jmt::RootHash;
use penumbra_storage::StateDelta;
use penumbra_storage::StateRead;
use penumbra_storage::StateWrite;
use penumbra_storage::Storage;
use tempfile;
use tokio;

/*
 * Migration tests.
 *
 * Node operators perform network upgrades by running a migration of
 * the chain state that preserve block height continuity. In order to
 * enable this, we need to have a way to commit changes to our merkle
 * tree, _without_ increasing its version number.
 *
 * With the addition of substores, we must cover the cases when migrations
 * accesses data located in substores.
 *
 * These integration tests enforce that a migration operation is able to
 * write to both the main store and any number of substores without incrementing
 * their version number.
 *
 */

#[tokio::test]
/// Test that we can commit to the main store without incrementing its version.
async fn test_simple_migration() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![];
    let storage = Storage::load(db_path, substore_prefixes).await?;

    let mut counter = 0;
    let num_writes = 10;

    for i in 0..num_writes {
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let key_1 = format!("key_{i}");
        let value_1 = format!("value_{i}").as_bytes().to_vec();
        delta.put_raw(key_1.clone(), value_1.clone());

        let _ = storage.commit(delta).await?;
        // Check that we can read the values back out.
        let snapshot = storage.latest_snapshot();

        let retrieved_value = snapshot.get_raw(key_1.as_str()).await?.unwrap();
        assert_eq!(retrieved_value, value_1);
        counter += 1;
    }

    let old_global_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");
    let old_version = storage.latest_version();
    assert_eq!(old_version, counter - 1);

    /* ********************* */
    /* perform the migration */
    /* ********************* */
    let mut delta = StateDelta::new(storage.latest_snapshot());
    let key_root_2 = "migration".to_string();
    let value_root_2 = "migration data".as_bytes().to_vec();
    delta.put_raw(key_root_2, value_root_2);
    let new_global_root = storage.commit_in_place(delta).await?;
    let new_version = storage.latest_version();

    assert_ne!(
        old_global_root, new_global_root,
        "migration did not effect the root hash"
    );

    assert_eq!(old_version, new_version, "the version number has changed!");

    assert_eq!(counter, num_writes);

    Ok(())
}

#[tokio::test]
/// Test that we can commit to substores without incrementing their version.
async fn test_substore_migration() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["ibc/".to_string(), "dex/".to_string(), "misc/".to_string()];
    let all_substores = vec!["ibc/".to_string(), "dex/".to_string(), "".to_string()];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    let mut counter = 0;
    let num_writes = 10;

    // Write to every substore, multiple times and check that we can read the values back out.
    for i in 0..num_writes {
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let mut keys: Vec<String> = vec![];
        let mut values: Vec<Vec<u8>> = vec![];
        for substore in all_substores.iter() {
            let key = format!("{substore}key_{i}");
            let value = format!("{substore}value_{i}").as_bytes().to_vec();
            tracing::debug!(?key, "initializing substore {substore} with key-value pair");
            delta.put_raw(key.clone(), value.clone());
            keys.push(key);
            values.push(value);
        }

        let _ = storage.commit(delta).await?;
        let snapshot = storage.latest_snapshot();

        for (key, value) in keys.iter().zip(values.iter()) {
            let retrieved_value = snapshot.get_raw(key.as_str()).await?.unwrap();
            assert_eq!(retrieved_value, *value);
        }

        counter += 1;
    }
    assert_eq!(counter, num_writes);

    let premigration_snapshot = storage.latest_snapshot();
    let mut old_root_hashes: Vec<RootHash> = vec![];
    for substore in all_substores.iter() {
        let root_hash = premigration_snapshot
            .prefix_root_hash(substore.as_str())
            .await
            .expect("prefix exists");
        old_root_hashes.push(root_hash);
    }

    let old_substore_versions: Vec<jmt::Version> = all_substores
        .clone()
        .into_iter()
        .map(|prefix| {
            let old_version = premigration_snapshot
                .prefix_version(prefix.as_str())
                .expect("prefix exists");
            old_version.expect("substore is initialized")
        })
        .collect();

    let old_version = storage.latest_version();
    assert_eq!(old_version, counter - 1); // -1 because we start at u64::MAX
    drop(premigration_snapshot);

    /* ******************************* */
    /*      perform the migration      */
    /* (write a key in every substore) */
    /* ******************************* */
    let mut delta = StateDelta::new(storage.latest_snapshot());

    // Start by writing a key in every substore, including the main store.
    for substore in all_substores.iter() {
        let key = format!("{substore}migration", substore = substore);
        let value = format!("{substore}migration data", substore = substore)
            .as_bytes()
            .to_vec();
        tracing::debug!(?key, "migration: writing to substore {substore}");
        delta.put_raw(key, value);
    }

    // Commit the migration.
    let _ = storage.commit_in_place(delta).await?;

    // Note(erwan): when we perform a commit in-place, we do not update the
    // snapshot cache. This means that querying `Storage::latest_snapshot()`
    // will return a now-stale view of the state.
    storage.release().await;
    let storage = Storage::load(db_path, substore_prefixes).await?;

    let postmigration_snapshot = storage.latest_snapshot();
    let new_version = storage.latest_version();

    assert_eq!(
        old_version, new_version,
        "the global version number has changed!"
    );

    // Check that the root hash for every substore has changed.
    let mut new_root_hashes: Vec<RootHash> = vec![];
    for substore in all_substores.iter() {
        let root_hash = postmigration_snapshot
            .prefix_root_hash(substore.as_str())
            .await
            .expect("prefix exists");
        new_root_hashes.push(root_hash);
    }

    old_root_hashes
        .iter()
        .zip(new_root_hashes.iter())
        .zip(all_substores.iter())
        .for_each(|((old, new), substore)| {
            assert_ne!(
                old, new,
                "migration did not effect the root hash for substore {substore}",
            );
        });

    // Check that the version number for every substore has NOT changed.
    let new_substore_versions: Vec<jmt::Version> = all_substores
        .clone()
        .into_iter()
        .map(|prefix| {
            let new_version = postmigration_snapshot
                .prefix_version(prefix.as_str())
                .expect("prefix exists");
            new_version.expect("substore is initialized")
        })
        .collect();

    old_substore_versions
        .iter()
        .zip(new_substore_versions.iter())
        .zip(all_substores.iter())
        .for_each(|((old, new), substore)| {
            assert_eq!(
                old, new,
                "the version number for substore {substore} has changed!",
            );
        });
    Ok(())
}
