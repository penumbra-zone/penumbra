#![cfg(feature = "migration")]
use cnidarium::StateDelta;
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
 * - prop_test_substore_migration: property-based testing of the migration operation.
 *
 * Each test has the following pattern:
 * Operation:
 *              Write a collection of keys, incrementing the version number at each step.
 * Checks:
 * - Check that the version number has incremented.
 * - Check that the keys are present in the latest snapshot.
 * - Check that the keys have valid proofs.
 * Operation:
 *              Perform a migration, writing/removing a key in the main store and/or substores.
 * - Check that the version number has not changed.
 * - Check that the root hash for the main store and/or substores has changed.
 * - Check that the migration key is present in the latest snapshot.
 * - Check that the migration key has a valid proof.
 * - Check that the migration key has the expected value.
 * Operation:
 *              Write a new collection of keys, incrementing the version number at each step.
 * Checks:
 * - Check that the version number has incremented.
 * - Check that the new keys are present in the latest snapshot.
 * - Check that the new keys have valid proofs.
 * - Check that the new keys have the expected values.
 * Operation:
 *              Try to generate proofs for keys that are NOT present in the jmt
 * Checks:
 * - Check that no value is returned for the keys.
 * - Check that the nonexistence proofs are valid.
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

    /* ************************ */
    /*      write some keys     */
    /* ************************ */
    let mut kvs = vec![];
    for i in 0..num_ops {
        /* write some value at version `i` */
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let key = format!("key_{i}");
        let value = format!("value_{i}").as_bytes().to_vec();
        delta.put_raw(key.clone(), value.clone());
        let root_hash = storage.commit(delta).await?;

        tracing::info!(%key, ?root_hash, version = %i, "committed key-value pair");

        kvs.push((key, value));
        counter += 1;
    }

    assert_eq!(counter, num_ops);
    counter = 0;

    // We don't _need_ to toss the storage instance, but let's be
    // extra careful and make sure that we can load the storage.
    storage.release().await;
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;
    let premigration_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");

    for (i, (key, value)) in kvs.clone().into_iter().enumerate() {
        let snapshot = storage.latest_snapshot();
        let (some_value, proof) = snapshot.get_with_proof(key.as_bytes().to_vec()).await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        let merkle_path = MerklePath {
            key_path: vec![key],
        };
        let merkle_root = MerkleRoot {
            hash: premigration_root.0.to_vec(),
        };

        proof
            .verify_membership(
                &MAIN_STORE_PROOF_SPEC,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!(?e, key_index = ?i, "proof verification failed"))
            .expect("membership proof verifies");

        counter += 1;
    }

    assert_eq!(counter, num_ops);
    counter = 0;

    let old_version = storage.latest_version();
    assert_eq!(old_version, num_ops - 1);

    /* ********************* */
    /* perform the migration */
    /* ********************* */
    let mut delta = StateDelta::new(storage.latest_snapshot());
    let migration_key = "banana".to_string();
    let migration_value = "a good fruit".as_bytes().to_vec();
    delta.put_raw(migration_key.clone(), migration_value.clone());
    let postmigration_root = storage.commit_in_place(delta).await?;

    // We have to reload the storage instance to get the latest snapshot.
    storage.release().await;
    let storage = Storage::load(db_path, substore_prefixes).await?;

    let new_version = storage.latest_version();

    assert_ne!(
        premigration_root, postmigration_root,
        "migration should change the root hash"
    );
    assert_eq!(
        old_version, new_version,
        "the post-migration version number should not change"
    );

    /* ************************ */
    /*   check the migration    */
    /* ************************ */
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
        hash: postmigration_root.0.to_vec(),
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

    /* ************************ */
    /*      write new keys      */
    /* ************************ */
    for i in num_ops..num_ops * 2 {
        /* write some value at version `i` */
        let mut delta = StateDelta::new(storage.latest_snapshot());
        let key = format!("key_{i}");
        let value = format!("value_{i}").as_bytes().to_vec();
        delta.put_raw(key.clone(), value.clone());
        let root_hash = storage.commit(delta).await?;

        tracing::info!(%key, ?root_hash, version = %i, "committed key-value pair");

        kvs.push((key, value));
        counter += 1;
    }

    assert_eq!(counter, num_ops);
    counter = 0;

    let final_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");

    for (i, (key, value)) in kvs.clone().into_iter().enumerate() {
        let snapshot = storage.latest_snapshot();
        let (some_value, proof) = snapshot.get_with_proof(key.as_bytes().to_vec()).await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        let merkle_path = MerklePath {
            key_path: vec![key],
        };
        let merkle_root = MerkleRoot {
            hash: final_root.0.to_vec(),
        };

        proof
            .verify_membership(
                &MAIN_STORE_PROOF_SPEC,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!(?e, key_index = ?i, "proof verification failed"))
            .expect("membership proof verifies");

        counter += 1;
    }

    assert_eq!(counter, num_ops * 2);

    /* ****************************** */
    /*      read nonexistent keys     */
    /* ****************************** */
    let final_snapshot = storage.latest_snapshot();
    let final_root = final_snapshot.root_hash().await.expect("infaillible");

    let key = format!("nonexistent_key");
    let (some_value, proof) = final_snapshot
        .get_with_proof(key.as_bytes().to_vec())
        .await?;
    assert!(some_value.is_none());
    let merkle_path = MerklePath {
        key_path: vec![key],
    };
    let merkle_root = MerkleRoot {
        hash: final_root.0.to_vec(),
    };

    proof
        .verify_non_membership(&MAIN_STORE_PROOF_SPEC, merkle_root, merkle_path)
        .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
        .expect("nonmembership proof verifies");

    Ok(())
}

#[tokio::test]
/// Test that we can commit to substores without incrementing their version.
async fn test_substore_migration() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["ibc".to_string(), "dex".to_string(), "misc".to_string()];
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    let mut counter = 0;
    let num_ops_per_substore = 10;

    let mut kvs = vec![];

    /* ************************ */
    /*      write some keys     */
    /*     in every substore    */
    /* ************************ */
    for i in 0..num_ops_per_substore {
        let mut delta = StateDelta::new(storage.latest_snapshot());
        for substore in substore_prefixes.iter() {
            let key = format!("{substore}/key_{i}");
            let value = format!("{substore}value_{i}").as_bytes().to_vec();
            kvs.push((key.clone(), value.clone()));
            tracing::debug!(?key, "initializing substore {substore} with key-value pair");
            delta.put_raw(key.clone(), value.clone());
        }

        let root_hash = storage.commit(delta).await?;
        tracing::info!(?root_hash, version = %i, "committed key-value pair");
        counter += 1;
    }
    let num_versions_pre_migration = counter;
    assert_eq!(counter, num_ops_per_substore);
    counter = 0;

    // We don't _need_ to toss the storage instance, but let's be
    // extra careful and make sure that things work if we reload it.
    storage.release().await;
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    let premigration_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");

    for (i, (key, value)) in kvs.clone().into_iter().enumerate() {
        tracing::debug!(?key, "checking key-value pair");
        let snapshot = storage.latest_snapshot();
        let (some_value, proof) = snapshot.get_with_proof(key.as_bytes().to_vec()).await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        // We split the key into its substore prefix and the key itself.
        let merkle_path = MerklePath {
            key_path: key.split('/').map(|s| s.to_string()).collect(),
        };
        let merkle_root = MerkleRoot {
            hash: premigration_root.0.to_vec(),
        };

        proof
            .verify_membership(
                &FULL_PROOF_SPECS,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!(?e, key_index = ?i, "proof verification failed"))
            .expect("membership proof verifies");

        counter += 1;
    }

    assert_eq!(
        counter,
        substore_prefixes.len() as u64 * num_ops_per_substore
    );

    let premigration_snapshot = storage.latest_snapshot();
    let mut old_root_hashes: Vec<RootHash> = vec![];
    for substore in substore_prefixes.iter() {
        let root_hash = premigration_snapshot
            .prefix_root_hash(substore.as_str())
            .await
            .expect("prefix exists");
        old_root_hashes.push(root_hash);
    }

    let old_substore_versions: Vec<jmt::Version> = substore_prefixes
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
    assert_eq!(old_version, num_versions_pre_migration - 1); // -1 because we start at u64::MAX
    let premigration_root_hash = premigration_snapshot
        .root_hash()
        .await
        .expect("infaillible");
    drop(premigration_snapshot);

    /* ******************************* */
    /*      perform the migration      */
    /* (write a key in every substore) */
    /* ******************************* */
    let mut delta = StateDelta::new(storage.latest_snapshot());
    let mut migration_kvs = vec![];

    // Start by writing a key in every substore, including the main store.
    for substore in substore_prefixes.iter() {
        let key = format!("{substore}/banana", substore = substore);
        let value = format!("{substore}", substore = substore)
            .as_bytes()
            .to_vec();
        tracing::debug!(?key, "migration: writing to substore {substore}");
        delta.put_raw(key.clone(), value.clone());
        migration_kvs.push((key, value));
    }

    // Commit the migration.
    let _ = storage.commit_in_place(delta).await?;

    /* ************************ */
    /*   check the migration    */
    /* ************************ */
    // Overview: We just wrote a key in every substore. Now we want to perform increasingly
    // complex checks to ensure that the migration was successful.
    // 1. Check that every root hash has changed
    // 2. Check that no version number has changed
    // 3. Check that we can read the migration key from every substore
    // 4. Check that the migration key has a valid proof
    // 5. Check that we can read every other key from every substore
    // 6. Check that every other key has a valid proof

    // We reload storage so that we can access the latest snapshot.
    // The snapshot cache is not updated when we commit in place.
    storage.release().await;
    let storage = Storage::load(db_path.clone(), substore_prefixes.clone()).await?;

    let postmigration_snapshot = storage.latest_snapshot();
    let new_version = storage.latest_version();

    assert_eq!(
        old_version, new_version,
        "the global version should not change"
    );

    let postmigration_root_hash = postmigration_snapshot
        .root_hash()
        .await
        .expect("infaillible");

    assert_ne!(premigration_root_hash, postmigration_root_hash);

    // Check that the root hash for every substore has changed.
    let mut new_root_hashes: Vec<RootHash> = vec![];
    for substore in substore_prefixes.iter() {
        let root_hash = postmigration_snapshot
            .prefix_root_hash(substore.as_str())
            .await
            .expect("prefix exists");
        new_root_hashes.push(root_hash);
    }

    old_root_hashes
        .iter()
        .zip(new_root_hashes.iter())
        .zip(substore_prefixes.iter())
        .for_each(|((old, new), substore)| {
            assert_ne!(
                old, new,
                "migration did not effect the root hash for substore {substore}",
            );
            let substore_version = postmigration_snapshot
                .prefix_version(substore.as_str())
                .expect("prefix exists")
                .unwrap();
            assert_eq!(
                substore_version,
                num_ops_per_substore - 1,
                "substore version should not change"
            );
        });

    // Check that the version number for every substore has NOT changed.
    let new_substore_versions: Vec<jmt::Version> = substore_prefixes
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
        .zip(substore_prefixes.iter())
        .for_each(|((old, new), substore)| {
            assert_eq!(
                old, new,
                "the version number for substore {substore} has changed!",
            );
        });

    // Check that the migration key is present in the latest snapshot.
    for (migration_key, migration_value) in migration_kvs.clone().into_iter() {
        let (some_value, proof) = postmigration_snapshot
            .get_with_proof(migration_key.as_bytes().to_vec())
            .await?;
        let retrieved_value = some_value.expect("migration key is found in the latest snapshot");
        assert_eq!(retrieved_value, migration_value);

        let merkle_path = MerklePath {
            key_path: migration_key.split('/').map(|s| s.to_string()).collect(),
        };
        let merkle_root = MerkleRoot {
            hash: postmigration_root_hash.0.to_vec(),
        };

        proof
            .verify_membership(
                &FULL_PROOF_SPECS,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
            .expect("membership proof verifies");
    }

    // Check that every other key is still present in the latest snapshot.
    for (key, value) in kvs.clone().into_iter() {
        let (some_value, proof) = postmigration_snapshot
            .get_with_proof(key.as_bytes().to_vec())
            .await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        let merkle_path = MerklePath {
            key_path: key.split('/').map(|s| s.to_string()).collect(),
        };
        let merkle_root = MerkleRoot {
            hash: postmigration_root_hash.0.to_vec(),
        };

        proof
            .verify_membership(
                &FULL_PROOF_SPECS,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
            .expect("membership proof verifies");
    }

    /* ************************ */
    /*      write some keys     */
    /*     in every substore    */
    /*        ... again ...     */
    /* ************************ */
    counter = 0;
    for i in 0..num_ops_per_substore {
        let mut delta = StateDelta::new(storage.latest_snapshot());
        for substore in substore_prefixes.iter() {
            let key = format!("{substore}/key_{i}");
            let value = format!("{substore}value_{i}").as_bytes().to_vec();
            kvs.push((key.clone(), value.clone()));
            tracing::debug!(?key, "initializing substore {substore} with key-value pair");
            delta.put_raw(key.clone(), value.clone());
        }

        let root_hash = storage.commit(delta).await?;
        tracing::info!(?root_hash, version = %i, "committed key-value pair");
        counter += 1;
    }
    assert_eq!(counter, num_ops_per_substore);
    counter = 0;

    let final_root = storage
        .latest_snapshot()
        .root_hash()
        .await
        .expect("infaillible");

    for (i, (key, value)) in kvs.clone().into_iter().enumerate() {
        tracing::debug!(?key, "checking key-value pair");
        let snapshot = storage.latest_snapshot();
        let (some_value, proof) = snapshot.get_with_proof(key.as_bytes().to_vec()).await?;
        let retrieved_value = some_value.expect("key is found in the latest snapshot");
        assert_eq!(retrieved_value, value);

        // We split the key into its substore prefix and the key itself.
        let merkle_path = MerklePath {
            key_path: key.split('/').map(|s| s.to_string()).collect(),
        };
        let merkle_root = MerkleRoot {
            hash: final_root.0.to_vec(),
        };

        proof
            .verify_membership(
                &FULL_PROOF_SPECS,
                merkle_root,
                merkle_path,
                retrieved_value,
                0,
            )
            .map_err(|e| tracing::error!(?e, key_index = ?i, "proof verification failed"))
            .expect("membership proof verifies");

        counter += 1;
    }

    assert_eq!(
        counter,
        // For each substore, we wrote `num_ops_per_substore` keys twice.
        substore_prefixes.len() as u64 * num_ops_per_substore * 2
    );

    /* ****************************** */
    /*      read nonexistent keys     */
    /* ****************************** */
    for (idx, substore) in substore_prefixes.iter().enumerate() {
        let key = format!("{substore}/nonexistent_key_{idx}");
        let (some_value, proof) = postmigration_snapshot
            .get_with_proof(key.as_bytes().to_vec())
            .await?;
        assert!(some_value.is_none());
        let merkle_path = MerklePath {
            key_path: key.split('/').map(|s| s.to_string()).collect(),
        };
        let merkle_root = MerkleRoot {
            hash: final_root.0.to_vec(),
        };

        proof
            .verify_non_membership(&FULL_PROOF_SPECS, merkle_root, merkle_path)
            .map_err(|e| tracing::error!("proof verification failed: {:?}", e))
            .expect("nonmembership proof verifies");
    }

    Ok(())
}

#[cfg(feature = "migration-proptests")]
mod proptests {
    use proptest::{
        arbitrary::any,
        prelude::prop,
        prop_assert, prop_assert_eq, prop_assert_ne, prop_oneof,
        strategy::{BoxedStrategy, Just, Strategy},
        test_runner::{FileFailurePersistence, TestCaseError},
    };
    use sha2::Sha256;
    use std::{
        collections::BTreeMap,
        fmt::{Debug, Display, Formatter},
        path::PathBuf,
    };
    use test_strategy::proptest;

    use cnidarium::{StateDelta, StateRead, StateWrite as _, Storage};
    use ibc_types::core::commitment::{MerklePath, MerkleRoot};

    use crate::{FULL_PROOF_SPECS, MAIN_STORE_PROOF_SPEC};

    struct ReferenceStore {
        final_kv: BTreeMap<StorageKey, Option<Vec<u8>>>,
    }

    impl ReferenceStore {
        fn new() -> Self {
            Self {
                final_kv: BTreeMap::new(),
            }
        }

        fn execute(&mut self, op: Operation) {
            match op {
                Operation::Insert(key, value) => {
                    if key.path() == "" {
                        panic!("empty key");
                    }
                    self.final_kv.insert(key, Some(value));
                }
                Operation::Delete(key) => {
                    self.final_kv.insert(key, None);
                }
            }
        }
    }

    fn prefix_list() -> Vec<String> {
        vec![
            "".to_string(),
            "dex".to_string(),
            "ibc".to_string(),
            "misc".to_string(),
            "staking".to_string(),
        ]
    }

    fn substore_list() -> Vec<String> {
        vec![
            "dex".to_string(),
            "ibc".to_string(),
            "misc".to_string(),
            "staking".to_string(),
        ]
    }

    fn char_except_slash() -> impl Strategy<Value = char> {
        any::<char>().prop_filter("Exclude '/'", |c| *c != '/')
    }

    fn valid_key_strategy() -> impl Strategy<Value = String> {
        (
            char_except_slash(),
            proptest::collection::vec(any::<char>(), 0..=999),
        )
            .prop_map(|(first_char, mut vec)| {
                vec.insert(0, first_char);
                vec.into_iter().collect()
            })
    }

    fn storage_key_strategy() -> BoxedStrategy<StorageKey> {
        let prefixes = prefix_list();
        let substore_strategies: Vec<BoxedStrategy<_>> =
            prefixes.into_iter().map(|s| Just(s).boxed()).collect();

        let substore_strategy = prop::strategy::Union::new_weighted(
            substore_strategies.into_iter().map(|s| (1, s)).collect(),
        );

        let key_strategy = valid_key_strategy();

        (substore_strategy, key_strategy)
            .prop_map(|(substore, key)| StorageKey::new(substore, key))
            .boxed()
    }

    fn value_strategy() -> impl Strategy<Value = Vec<u8>> {
        // Generate a random byte array of length 1..10
        // The values don't actually matter for the tests.
        prop::collection::vec(any::<u8>(), 1..10)
    }

    fn operation_strategy() -> impl Strategy<Value = Operation> {
        let insert_strategy = (storage_key_strategy(), value_strategy())
            .prop_map(|(key, value)| Operation::Insert(key, value));

        let delete_strategy = storage_key_strategy().prop_map(Operation::Delete);

        prop_oneof![insert_strategy, delete_strategy,]
    }

    fn insert_strategy() -> impl Strategy<Value = Operation> {
        let insert_strategy = (storage_key_strategy(), value_strategy())
            .prop_map(|(key, value)| Operation::Insert(key, value));

        prop_oneof![insert_strategy]
    }

    fn insert_ops_strategy() -> impl Strategy<Value = Vec<Operation>> {
        prop::collection::vec(insert_strategy(), 0..10000)
    }

    fn operations_strategy() -> impl Strategy<Value = Vec<Operation>> {
        prop::collection::vec(operation_strategy(), 0..10000)
    }

    fn execute_transcript<S: StateRead>(
        phase: &str,
        reference_store: &mut ReferenceStore,
        delta: &mut StateDelta<S>,
        transcript: Vec<Operation>,
    ) {
        for op in transcript {
            reference_store.execute(op.clone());
            let storage_key = op.key();
            let key_hash = storage_key.hash();
            let key_path = storage_key.path();

            tracing::debug!(
                prefix = storage_key.prefix(),
                key = storage_key.truncated_key(),
                ?key_hash,
                ?op,
                ?phase,
            );

            match op {
                Operation::Insert(_key, value) => {
                    delta.put_raw(key_path, value);
                }
                Operation::Delete(_key) => {
                    delta.delete(key_path);
                }
            }
        }
    }

    async fn check_proofs(
        state: cnidarium::Snapshot,
        reference_store: &ReferenceStore,
        root_hash: jmt::RootHash,
        phase: &str,
    ) -> Result<(), TestCaseError> {
        for (storage_key, reference_value) in reference_store.final_kv.iter() {
            let key_hash = storage_key.hash();
            let key_path = storage_key.path();

            tracing::debug!(
                prefix = storage_key.prefix(),
                key = storage_key.truncated_key(),
                ?key_hash,
                ?phase,
                "checking proofs"
            );
            let result_proof = state
                .get_with_proof(storage_key.encode_path())
                .await
                .map_err(|e| tracing::error!(?e, "get_with_proof failed"));

            prop_assert!(result_proof.is_ok(), "can get with proof");

            let (retrieved_value, proof) = result_proof.expect("can get with proof");
            prop_assert_eq!(&retrieved_value, reference_value);

            let merkle_path = storage_key.merkle_path();
            let merkle_root = MerkleRoot {
                hash: root_hash.0.to_vec(),
            };
            let specs = storage_key.proof_spec();
            let key_hash = jmt::KeyHash::with::<Sha256>(&key_path);

            tracing::debug!(
                prefix = storage_key.prefix(),
                num_proofs = proof.proofs.len(),
                spec_len = specs.len(),
                ?key_hash,
                truncated_key = storage_key.truncated_key(),
                is_existence = reference_value.is_some(),
                "proof verification"
            );

            let proof_verifies = if let Some(value) = retrieved_value.clone() {
                proof
                    .verify_membership(&specs, merkle_root, merkle_path, value, 0)
                    .map_err(|e| tracing::error!(?e, "existence proof failed"))
            } else {
                proof
                    .verify_non_membership(&specs, merkle_root, merkle_path)
                    .map_err(|e| tracing::error!(?e, "nonexistence proof failed"))
            };

            prop_assert!(proof_verifies.is_ok());
        }
        Ok(())
    }

    /// Implements a standard migration test which consists of three phases:
    /// - premigration: write and delete keys, check ex/nex proofs
    /// - migration: write and delete keys using a migration commit, check ex/nex proofs
    /// - postmigration: write new keys, check ex/nex proofs
    async fn standard_migration_test(
        db_path: PathBuf,
        premigration_transcript: Vec<Operation>,
        migration_transcript: Vec<Operation>,
        postmigration_transcript: Vec<Operation>,
    ) -> Result<(), TestCaseError> {
        let substore_prefixes = substore_list();

        let premigration_len = premigration_transcript.len();
        let migration_len = migration_transcript.len();
        let postmigration_len = postmigration_transcript.len();
        let total_ops = premigration_len + migration_len + postmigration_len;

        tracing::info!(
            premigration_len,
            migration_len,
            postmigration_len,
            total_ops,
            "starting test"
        );

        let storage = Storage::load(db_path.clone(), substore_prefixes.clone())
            .await
            .expect("Failed to load storage");
        // `ReferenceStore` is an in-memory store that tracks the latest value for each key
        // To do this, the store execute each operation in the transcript and tracks the final value.
        // It serves as a source of truth to compare the storage against.
        let mut reference_store = ReferenceStore::new();

        // Premigration: write and delete keys
        let mut premigration_delta = StateDelta::new(storage.latest_snapshot());

        execute_transcript(
            "premigration",
            &mut reference_store,
            &mut premigration_delta,
            premigration_transcript,
        );

        let premigration_root_hash = storage
            .commit(premigration_delta)
            .await
            .expect("can commit premigration");
        let premigration_version = storage.latest_version();
        prop_assert_eq!(premigration_version, 0, "premigration version should be 0");

        tracing::info!(
            ?premigration_root_hash,
            premigration_len,
            "premigration operations have been committed"
        );

        let premigration_snapshot = storage.latest_snapshot();
        let _ = check_proofs(
            premigration_snapshot,
            &reference_store,
            premigration_root_hash,
            "premigration",
        )
        .await?;

        // Migration: write and delete keys
        let mut migration_delta = StateDelta::new(storage.latest_snapshot());
        execute_transcript(
            "migration",
            &mut reference_store,
            &mut migration_delta,
            migration_transcript,
        );

        let migration_root_hash = storage
            .commit_in_place(migration_delta)
            .await
            .expect("can commit migration");
        let migration_version = storage.latest_version();
        prop_assert_eq!(migration_version, 0, "migration version should be 0");
        prop_assert_ne!(
            migration_root_hash,
            premigration_root_hash,
            "migration root hash should be different than the premigration root hash"
        );

        storage.release().await;
        let storage = Storage::load(db_path.clone(), substore_prefixes.clone())
            .await
            .expect("can reload storage");

        tracing::info!(
            ?migration_root_hash,
            migration_len,
            "migration operations have been committed"
        );

        let migration_snapshot = storage.latest_snapshot();

        let _ = check_proofs(
            migration_snapshot,
            &reference_store,
            migration_root_hash,
            "migration",
        )
        .await?;

        // We toss the storage instance and reload it.
        storage.release().await;
        let storage = Storage::load(db_path.clone(), substore_prefixes.clone())
            .await
            .expect("can reload storage");

        // Post-migration: write new keys!
        let mut postmigration_delta = StateDelta::new(storage.latest_snapshot());
        execute_transcript(
            "postmigration",
            &mut reference_store,
            &mut postmigration_delta,
            postmigration_transcript,
        );

        let postmigration_root_hash = storage
            .commit(postmigration_delta)
            .await
            .expect("can commit postmigration");
        tracing::info!(
            ?postmigration_root_hash,
            num_ops = postmigration_len,
            "postmigration operations have been committed"
        );

        let postmigration_version = storage.latest_version();
        prop_assert_eq!(
            postmigration_version,
            1,
            "postmigration version should be 1"
        );
        prop_assert_ne!(
            migration_root_hash,
            postmigration_root_hash,
            "postmigration root hash should be different than the migration root hash"
        );

        let post_migration_snapshot = storage.latest_snapshot();

        let _ = check_proofs(
            post_migration_snapshot,
            &reference_store,
            postmigration_root_hash,
            "postmigration",
        )
        .await?;
        Ok(())
    }

    #[proptest(async = "tokio", cases = 100, failure_persistence = Some(Box::new(FileFailurePersistence::WithSource("regressions"))))]
    async fn test_migration_substores(
        #[strategy(operations_strategy())] premigration_transcript: Vec<Operation>,
        #[strategy(operations_strategy())] migration_transcript: Vec<Operation>,
        #[strategy(operations_strategy())] postmigration_transcript: Vec<Operation>,
        #[strategy(insert_ops_strategy())] nonexistence_keys: Vec<Operation>,
    ) {
        let _ = tracing_subscriber::fmt::try_init();

        let tmpdir = tempfile::tempdir().expect("Failed to create a temp dir");
        let db_path = tmpdir.into_path();
        let _ = standard_migration_test(
            db_path.clone(),
            premigration_transcript,
            migration_transcript,
            postmigration_transcript,
        )
        .await;

        let storage = Storage::load(db_path.clone(), substore_list())
            .await
            .expect("can reload storage");

        let postmigration_root_hash = storage
            .latest_snapshot()
            .root_hash()
            .await
            .expect("infaillible");

        // Check random keys that should not exist
        for op in nonexistence_keys {
            let storage_key = op.key();
            let key_hash = storage_key.hash();
            let key_path = storage_key.path();

            tracing::debug!(
                prefix = storage_key.prefix(),
                key = &storage_key.truncated_key(),
                ?key_hash,
                ?op,
                "nex: checking proofs"
            );

            let result_proof = storage
                .latest_snapshot()
                .get_with_proof(storage_key.encode_path())
                .await
                .map_err(|e| tracing::error!(?e, "nex: get_with_proof failed"));

            prop_assert!(result_proof.is_ok(), "can get with proof");

            let (retrieved_value, proof) = result_proof.expect("can get with proof");
            prop_assert!(retrieved_value.is_none(), "key should not exist");

            let merkle_path = storage_key.merkle_path();
            let merkle_root = MerkleRoot {
                hash: postmigration_root_hash.0.to_vec(),
            };
            let specs = storage_key.proof_spec();
            let key_hash = jmt::KeyHash::with::<Sha256>(&key_path);

            tracing::debug!(
                prefix = storage_key.prefix(),
                num_proofs = proof.proofs.len(),
                spec_len = specs.len(),
                ?key_hash,
                truncated_key = &storage_key.truncated_key(),
                "nex: proof verification"
            );

            let proof_verifies = proof
                .verify_non_membership(&specs, merkle_root, merkle_path)
                .map_err(|e| tracing::error!(?e, "nonexistence: nonexistence proof failed"));

            prop_assert!(proof_verifies.is_ok());
        }
    }

    #[derive(Clone)]
    enum Operation {
        Insert(StorageKey, Vec<u8>),
        Delete(StorageKey),
    }

    impl Operation {
        fn key(&self) -> &StorageKey {
            match self {
                Operation::Insert(key, _) => key,
                Operation::Delete(key) => key,
            }
        }
    }

    impl Debug for Operation {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Operation::Insert(_, _) => write!(f, "Insert"),
                Operation::Delete(_) => write!(f, "Delete"),
            }
        }
    }

    #[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
    struct StorageKey {
        prefix: String,
        key: String,
        full_path: String,
    }

    impl Display for StorageKey {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.path())
        }
    }

    impl Debug for StorageKey {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.full_path)
        }
    }

    impl StorageKey {
        fn new(prefix: String, key: String) -> Self {
            let full_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}/{}", prefix, key)
            };

            Self {
                prefix,
                key,
                full_path,
            }
        }

        fn truncated_key(&self) -> String {
            self.key.chars().take(5).collect()
        }

        fn hash(&self) -> jmt::KeyHash {
            jmt::KeyHash::with::<Sha256>(&self.full_path)
        }

        fn encode_path(&self) -> Vec<u8> {
            self.path().as_bytes().to_vec()
        }

        fn prefix(&self) -> &String {
            &self.prefix
        }

        fn is_main_store(&self) -> bool {
            self.prefix == ""
        }

        fn path(&self) -> String {
            self.full_path.clone()
        }

        fn merkle_path(&self) -> MerklePath {
            if self.is_main_store() {
                return MerklePath {
                    key_path: vec![self.key.clone()],
                };
            } else {
                MerklePath {
                    key_path: vec![self.prefix.clone(), self.key.clone()],
                }
            }
        }

        fn proof_spec(&self) -> Vec<ics23::ProofSpec> {
            if self.is_main_store() {
                MAIN_STORE_PROOF_SPEC.clone()
            } else {
                FULL_PROOF_SPECS.clone()
            }
        }
    }
}
