#![cfg(feature = "migration")]
use cnidarium::StateDelta;
use cnidarium::StateRead;
use cnidarium::StateWrite;
use cnidarium::Storage;
use ibc_types::core::commitment::MerklePath;
use ibc_types::core::commitment::MerkleRoot;
use jmt::RootHash;
use once_cell::sync::Lazy;
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
 * Testing menu:
 * - test_simple_migration: the most basic migration scenario where we write to the main store.
 * - test_substore_migration: a migration scenario where we write to the main store and substores.
 * - prop_migration: property-based testing of the migration operation.
 *
 * Each test has the following pattern:
 * - Write a collection of keys, incrementing the version number at each step.
 * - Check that the version number has incremented.
 * - Check that the keys are present in the latest snapshot.
 * - Check that the keys have valid proofs.
 * - Perform a migration, writing/removing a key in the main store and/or substores.
 * - Check that the version number has not changed.
 * - Check that the root hash for the main store and/or substores has changed.
 * - Check that the migration key is present in the latest snapshot.
 * - Check that the migration key has a valid proof.
 * - Check that the migration key has the expected value.
 * - Write a new collection of keys, incrementing the version number at each step.
 * - Check that the version number has incremented.
 * - Check that the new keys are present in the latest snapshot.
 * - Check that the new keys have valid proofs.
 * - Check that the new keys have the expected values.
 */

/// The proof specs for the main store.
pub static MAIN_STORE_PROOF_SPEC: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![cnidarium::ics23_spec()]);

/// The proof specs for keys located in substores (e.g. `ibc` keys)
pub static FULL_PROOF_SPECS: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![cnidarium::ics23_spec(), cnidarium::ics23_spec()]);

#[tokio::test]
/// Test that we can commit to the main store without incrementing its version.
async fn test_simple_migration() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    let mut counter = 0;
    let num_ops = 10;

    let mut kvs = vec![];
    let mut roots = vec![];
    for i in 0..num_ops {
        /* write some value at version `i` */
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let key = format!("key_{i}");
        let value = format!("value_{i}").as_bytes().to_vec();
        delta.put_raw(key.clone(), value.clone());
        let root_hash = storage.commit(delta).await?;

        kvs.push((key, value));
        roots.push(root_hash);
        counter += 1;
    }

    assert_eq!(counter, num_ops);

    counter = 0;

    // We don't _need_ to toss the storage instance, but let's be
    // extra careful and make sure that we can load the storage.
    storage.release().await;
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    for (i, (key, value)) in kvs.clone().into_iter().enumerate() {
        let root_i = roots[i];

        let snapshot = storage.latest_snapshot();
        let (some_value, proof) = snapshot.get_with_proof(key.as_bytes().to_vec()).await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        let merkle_path = MerklePath {
            key_path: vec![key],
        };
        let merkle_root = MerkleRoot {
            hash: root_i.0.to_vec(),
        };

        proof
            .verify_membership(
                &MAIN_STORE_PROOF_SPEC,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
            .expect("membership proof verifies");

        counter += 1;
    }

    assert_eq!(counter, num_ops);

    let premigration_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");
    let old_version = storage.latest_version();
    assert_eq!(old_version, num_ops - 1);

    /* ********************* */
    /* perform the migration */
    /* ********************* */
    let mut delta = StateDelta::new(storage.latest_snapshot());
    let migration_key = "migration".to_string();
    let migration_value = "migration data".as_bytes().to_vec();
    delta.put_raw(migration_key.clone(), migration_value.clone());
    let new_global_root = storage.commit_in_place(delta).await?;

    storage.release().await;
    let storage = Storage::load(db_path, substore_prefixes).await?;

    let new_version = storage.latest_version();

    assert_ne!(
        premigration_root, new_global_root,
        "migration did not effect the root hash"
    );

    assert_eq!(old_version, new_version, "the version number has changed!");

    assert_eq!(counter, num_ops);

    let (some_value, proof) = storage
        .latest_snapshot()
        .get_with_proof(migration_key.as_bytes().to_vec())
        .await?;

    let retrieved_value = some_value.expect("migration key is found in the latest snapshot");
    assert_eq!(retrieved_value, migration_value);

    let merkle_path = MerklePath {
        key_path: vec![migration_key],
    };
    let merkle_root = MerkleRoot {
        hash: new_global_root.0.to_vec(),
    };

    proof
        .verify_membership(
            &MAIN_STORE_PROOF_SPEC,
            merkle_root,
            merkle_path,
            retrieved_value,
            0,
        )
        .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
        .expect("membership proof verifies");

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
            let key = format!("{substore}/key_{i}");
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
        let key = format!("{substore}/migration", substore = substore);
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
