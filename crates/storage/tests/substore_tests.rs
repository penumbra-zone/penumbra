use jmt::RootHash;
use penumbra_storage::StateDelta;
use penumbra_storage::StateRead;
use penumbra_storage::StateWrite;
use penumbra_storage::Storage;
use tempfile;
use tokio;

#[tokio::test]
#[should_panic]
/// Test that we cannot create a storage with an empty substore prefix.
async fn test_disallow_empty_prefix() -> () {
    let tmpdir = tempfile::tempdir().expect("creating a temporary directory works");
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![""].into_iter().map(|s| s.to_string()).collect();
    let _ = Storage::init(db_path, substore_prefixes).await.unwrap();
}

#[tokio::test]
/// Test that we can create a storage with multiple substores, that we can write to them, and that
/// we can read from them.
async fn test_substore_simple() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["prefix_a", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::init(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let key_a_1 = "prefix_a/key_1".to_string();
    let value_a_1 = "value_1a".as_bytes().to_vec();
    tracing::debug!(?key_a_1, ?value_a_1, "calling `put_raw`");
    delta.put_raw(key_a_1.clone(), value_a_1.clone());

    let key_root_1 = "key_1".to_string();
    let value_root_1 = "value_1".as_bytes().to_vec();
    tracing::debug!(?key_root_1, ?value_root_1, "calling `put_raw`");
    delta.put_raw(key_root_1.clone(), value_root_1.clone());

    tracing::debug!("committing first batch of writes");
    let global_root_hash_1 = storage.commit(delta).await?;

    tracing::debug!("checking that we can read the values back out");
    // Check that we can read the values back out.
    let snapshot = storage.latest_snapshot();

    let retrieved_value_a1 = snapshot.get_raw(key_a_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a1, value_a_1);

    let retrieved_value_root_1 = snapshot.get_raw(key_root_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_root_1, value_root_1);

    // Check that the prefix root hash is stored at the correct key.
    // One method looks up the key in the main store, the other creates a jmt over
    // the substore and looks up the root hash in that tree.
    let retrieved_prefix_a_root_hash = snapshot
        .get_raw("prefix_a")
        .await?
        .expect("key `prefix_a` should contain the substore root hash");
    assert_eq!(
        retrieved_prefix_a_root_hash.len(),
        global_root_hash_1.0.len()
    );

    let retrieved_prefix_a_root_hash = RootHash(retrieved_prefix_a_root_hash.try_into().unwrap());
    let prefix_a_root_hash = snapshot.prefix_root_hash("prefix_a").await?;
    assert_eq!(prefix_a_root_hash, retrieved_prefix_a_root_hash);
    let old_prefix_a_root_hash = prefix_a_root_hash;

    drop(snapshot);

    // Check that we can read new values from a prefixed substore, then check that
    // versioning works correctly, by making sure that we fetch the correct root hash.
    let key_a_2 = "prefix_a/key_2".to_string();
    let value_a_2 = "value_2a".as_bytes().to_vec();
    let mut delta = StateDelta::new(storage.latest_snapshot());
    delta.put_raw(key_a_2.clone(), value_a_2.clone());
    let global_root_hash_2 = storage.commit(delta).await?;

    // CHeck that we can read the new value back out.
    let snapshot = storage.latest_snapshot();
    let retrieved_value_a2 = snapshot.get_raw(key_a_2.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a2, value_a_2);
    let retrieved_value_a1 = snapshot.get_raw(key_a_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a1, value_a_1);

    // Retrieve the substore root hash again, and check that it has changed.
    assert_ne!(global_root_hash_1, global_root_hash_2); // sanity check.
    let prefix_a_root_hash = snapshot.prefix_root_hash("prefix_a").await?;
    let retrieved_prefix_a_root_hash = snapshot
        .get_raw("prefix_a")
        .await?
        .expect("prefix_a should contain the substore root hash");
    assert_eq!(
        prefix_a_root_hash,
        RootHash(retrieved_prefix_a_root_hash.try_into().unwrap())
    );
    assert_ne!(prefix_a_root_hash, old_prefix_a_root_hash);

    Ok(())
}

#[tokio::test]
/// Test that we can create a storage with multiple substores, that we can write to them, and that
/// we can read from them using prefix queries.
async fn test_substore_prefix_queries() -> anyhow::Result<()> {
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["prefix_a", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::init(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let key_a_1 = "prefix_a/key_1".to_string();
    let value_a_1 = "value_1a".as_bytes().to_vec();
    delta.put_raw(key_a_1.clone(), value_a_1.clone());

    let key_root_1 = "key_1".to_string();
    let value_root_1 = "value_1".as_bytes().to_vec();
    delta.put_raw(key_root_1.clone(), value_root_1.clone());

    let global_root_hash_1 = storage.commit(delta).await?;

    // Check that we can read the values back out.
    let snapshot = storage.latest_snapshot();

    let retrieved_value_a1 = snapshot.get_raw(key_a_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a1, value_a_1);

    let retrieved_value_root_1 = snapshot.get_raw(key_root_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_root_1, value_root_1);

    // Check that the prefix root hash is stored at the correct key.
    // One method looks up the key in the main store, the other creates a jmt over
    // the substore and looks up the root hash in that tree.
    let retrieved_prefix_a_root_hash = snapshot
        .get_raw("prefix_a")
        .await?
        .expect("prefix_a should have a root hash");
    assert_eq!(
        retrieved_prefix_a_root_hash.len(),
        global_root_hash_1.0.len()
    );

    let retrieved_prefix_a_root_hash = RootHash(retrieved_prefix_a_root_hash.try_into().unwrap());
    let prefix_a_root_hash = snapshot.prefix_root_hash("prefix_a").await?;
    assert_eq!(prefix_a_root_hash, retrieved_prefix_a_root_hash);
    let old_prefix_a_root_hash = prefix_a_root_hash;

    drop(snapshot);

    // Check that we can read new values from a prefixed substore, then check that
    // versioning works correctly, by making sure that we fetch the correct root hash.
    let key_a_2 = "prefix_a/key_2".to_string();
    let value_a_2 = "value_2a".as_bytes().to_vec();
    let mut delta = StateDelta::new(storage.latest_snapshot());
    delta.put_raw(key_a_2.clone(), value_a_2.clone());
    let global_root_hash_2 = storage.commit(delta).await?;

    // CHeck that we can read the new value back out.
    let snapshot = storage.latest_snapshot();
    let retrieved_value_a2 = snapshot.get_raw(key_a_2.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a2, value_a_2);
    let retrieved_value_a1 = snapshot.get_raw(key_a_1.as_str()).await?.unwrap();
    assert_eq!(retrieved_value_a1, value_a_1);

    // Retrieve the substore root hash again, and check that it has changed.
    assert_ne!(global_root_hash_1, global_root_hash_2); // sanity check.
    let prefix_a_root_hash = snapshot.prefix_root_hash("prefix_a").await?;
    let retrieved_prefix_a_root_hash = snapshot
        .get_raw("prefix_a")
        .await?
        .expect("prefix_a should have a root hash stored at this key");
    assert_eq!(
        prefix_a_root_hash,
        RootHash(retrieved_prefix_a_root_hash.try_into().unwrap())
    );
    assert_ne!(prefix_a_root_hash, old_prefix_a_root_hash);

    Ok(())
}
