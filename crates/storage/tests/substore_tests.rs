use ibc_types::core::commitment::MerklePath;
use ibc_types::core::commitment::MerkleRoot;
use jmt::RootHash;
use once_cell::sync::Lazy;
use penumbra_storage::StateDelta;
use penumbra_storage::StateRead;
use penumbra_storage::StateWrite;
use penumbra_storage::Storage;
use tempfile;
use tokio;
use tokio_stream::StreamExt;

#[tokio::test]
#[should_panic]
/// Test that we cannot create a storage with an empty substore prefix.
async fn test_disallow_empty_prefix() -> () {
    let tmpdir = tempfile::tempdir().expect("creating a temporary directory works");
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec![""].into_iter().map(|s| s.to_string()).collect();
    let _ = Storage::load(db_path, substore_prefixes).await.unwrap();
}

#[tokio::test]
async fn test_substore_proofs() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["ibc/", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::load(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let key_a_1 = "ibc/key_1".to_string();
    let value_a_1 = "value_1a".as_bytes().to_vec();
    delta.put_raw(key_a_1.clone(), value_a_1.clone());

    storage.commit(delta).await?;

    let snapshot = storage.latest_snapshot();

    pub static PENUMBRA_PROOF_SPECS: Lazy<Vec<ics23::ProofSpec>> = Lazy::new(|| {
        vec![
            penumbra_storage::ics23_spec(),
            penumbra_storage::ics23_spec(),
        ]
    });

    // check that we can verify proofs back to the root for the new value.
    let root = snapshot.root_hash().await?;
    let (retreived_value, proof) = snapshot.get_with_proof(key_a_1.into()).await?;
    assert_eq!(Some(value_a_1), retreived_value.clone());
    let merkle_path = MerklePath {
        key_path: vec!["ibc/".to_string(), "key_1".to_string()],
    };
    let merkle_root = MerkleRoot {
        hash: root.0.to_vec(),
    };
    proof.verify_membership(
        &PENUMBRA_PROOF_SPECS,
        merkle_root.clone(),
        merkle_path,
        retreived_value.unwrap(),
        0,
    )?;

    // check that non-existence proofs work
    let (retreived_value, nex_proof) = snapshot.get_with_proof("ibc/doesntexist".into()).await?;
    assert_eq!(retreived_value, None);

    tracing::debug!(?retreived_value, ?nex_proof, "got non-existence proof");

    let merkle_path = MerklePath {
        key_path: vec!["ibc/".to_string(), "doesntexist".to_string()],
    };
    nex_proof.verify_non_membership(&PENUMBRA_PROOF_SPECS, merkle_root, merkle_path)?;

    Ok(())
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
    let storage = Storage::load(db_path, substore_prefixes).await?;
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
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["prefix_a", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::load(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let mut all_kv = vec![];
    let mut kv_a = vec![];
    let mut kv_b = vec![];
    let mut kv_main = vec![];
    for i in 0..10 {
        let key_a_i = format!("prefix_a/key_{}", i);
        let value_a_i = format!("value_{}a", i).as_bytes().to_vec();
        delta.put_raw(key_a_i.clone(), value_a_i.clone());
        all_kv.push((key_a_i.clone(), value_a_i.clone()));
        kv_a.push((key_a_i, value_a_i));
    }

    for i in 0..10 {
        let key_b_i = format!("prefix_b/key_{}", i);
        let value_b_i = format!("value_{}b", i).as_bytes().to_vec();
        delta.put_raw(key_b_i.clone(), value_b_i.clone());
        all_kv.push((key_b_i.clone(), value_b_i.clone()));
        kv_b.push((key_b_i, value_b_i));
    }

    for i in 0..10 {
        let key_i = format!("key_{}", i);
        let value_i = format!("value_{}", i).as_bytes().to_vec();
        delta.put_raw(key_i.clone(), value_i.clone());
        all_kv.push((key_i.clone(), value_i.clone()));
        kv_main.push((key_i, value_i));
    }

    let _ = storage.commit(delta).await?;

    let snapshot = storage.latest_snapshot();
    let mut counter = 0;
    let query_prefix = "prefix_a";
    let mut range = snapshot.prefix_keys(query_prefix);
    while let Some(res) = range.next().await {
        let key = res?;
        let key = format!("prefix_a/{key}");
        if counter >= kv_a.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_a[counter].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_a.len(),
        "should have iterated over all keys (prefix_a)"
    );

    let mut counter = 0;
    let query_prefix = "prefix_b";
    let mut range = snapshot.prefix_keys(query_prefix);
    while let Some(res) = range.next().await {
        let key = res?;
        let key = format!("prefix_b/{key}");

        if counter >= kv_b.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_b[counter].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_b.len(),
        "should have iterated over all keys (prefix_b)"
    );

    let mut counter = 0;
    let query_prefix = "key";
    let mut range = snapshot.prefix_keys(query_prefix);
    while let Some(res) = range.next().await {
        let key = res?;
        tracing::debug!(?key, ?query_prefix, "iterating over keys");

        if counter >= kv_main.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_main[counter].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_main.len(),
        "should have iterated over all keys (main)"
    );

    Ok(())
}

#[tokio::test]
/// Test that `StateRead::prefix_keys` work as expected.
/// This test is similar to `test_substore_prefix_queries`, but uses `prefix_keys` instead of
/// `prefix_raw`.
async fn test_substore_prefix_keys() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["prefix_a", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::load(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let mut all_kv = vec![];
    let mut kv_a = vec![];
    let mut kv_b = vec![];
    let mut kv_main = vec![];
    for i in 0..10 {
        let key_a_i = format!("prefix_a/key_{}", i);
        let value_a_i = format!("value_{}a", i).as_bytes().to_vec();
        delta.put_raw(key_a_i.clone(), value_a_i.clone());
        all_kv.push((key_a_i.clone(), value_a_i.clone()));
        kv_a.push((key_a_i, value_a_i));
    }

    for i in 0..10 {
        let key_b_i = format!("prefix_b/key_{}", i);
        let value_b_i = format!("value_{}b", i).as_bytes().to_vec();
        delta.put_raw(key_b_i.clone(), value_b_i.clone());
        all_kv.push((key_b_i.clone(), value_b_i.clone()));
        kv_b.push((key_b_i, value_b_i));
    }

    for i in 0..10 {
        let key_i = format!("key_{}", i);
        let value_i = format!("value_{}", i).as_bytes().to_vec();
        delta.put_raw(key_i.clone(), value_i.clone());
        all_kv.push((key_i.clone(), value_i.clone()));
        kv_main.push((key_i, value_i));
    }

    let _ = storage.commit(delta).await?;

    let snapshot = storage.latest_snapshot();
    let mut counter = 0;
    let query_prefix = "prefix_a";
    let mut range = snapshot.prefix_keys(query_prefix);
    while let Some(res) = range.next().await {
        let key = res?;
        let key = format!("prefix_a/{key}");
        if counter >= kv_a.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_a[counter].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_a.len(),
        "should have iterated over all keys (prefix_a)"
    );

    let mut counter = 0;
    let query_prefix = "prefix_b";
    let mut range = snapshot.prefix_keys(query_prefix);
    while let Some(res) = range.next().await {
        let key = res?;
        let key = format!("prefix_b/{key}");

        if counter >= kv_b.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_b[counter].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_b.len(),
        "should have iterated over all keys (prefix_b)"
    );

    let mut counter = 0;
    let query_prefix = "key";
    let mut range = snapshot.prefix_raw(query_prefix);
    while let Some(res) = range.next().await {
        let (key, value) = res?;
        tracing::debug!(?key, ?query_prefix, "iterating over key/value pair");

        if counter >= kv_main.len() {
            tracing::debug!(?key, ?value, ?query_prefix, "unexpected key/value pair");
            panic!("prefix query returned too many entries")
        }

        let expected_key = kv_main[counter].0.clone();
        let expected_value = kv_main[counter].1.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        assert_eq!(value, expected_value, "value {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_main.len(),
        "should have iterated over all keys (main)"
    );

    Ok(())
}

#[tokio::test]
/// Test that `StateRead::nonverifiable_prefix_raw` works as expected.
/// This test is similar to `test_substore_prefix_queries`, but uses nonverifiable storage.
async fn test_substore_nv_prefix() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["prefix_a", "prefix_b", "prefix_c"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::load(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let mut all_kv = vec![];
    let mut kv_a = vec![];
    let mut kv_b = vec![];
    let mut kv_main = vec![];
    for i in 0..10 {
        let key_a_i = format!("prefix_a/key_{}", i);
        let value_a_i = format!("value_{}a", i);
        delta.nonverifiable_put_raw(key_a_i.as_bytes().to_vec(), value_a_i.as_bytes().to_vec());
        all_kv.push((key_a_i.clone(), value_a_i.clone()));
        kv_a.push((key_a_i, value_a_i));
    }

    for i in 0..10 {
        let key_b_i = format!("prefix_b/key_{}", i);
        let value_b_i = format!("value_{}b", i);
        delta.nonverifiable_put_raw(key_b_i.as_bytes().to_vec(), value_b_i.as_bytes().to_vec());
        all_kv.push((key_b_i.clone(), value_b_i.clone()));
        kv_b.push((key_b_i, value_b_i));
    }

    for i in 0..10 {
        let key_i = format!("key_{}", i);
        let value_i = format!("value_{}", i);
        delta.nonverifiable_put_raw(key_i.as_bytes().to_vec(), value_i.as_bytes().to_vec());
        all_kv.push((key_i.clone(), value_i.clone()));
        kv_main.push((key_i, value_i));
    }

    let _ = storage.commit(delta).await?;

    let snapshot = storage.latest_snapshot();
    let mut counter = 0;
    let query_prefix = "prefix_a".as_bytes();
    let mut range = snapshot.nonverifiable_prefix_raw(query_prefix);
    while let Some(res) = range.next().await {
        let (raw_key, raw_value) = res?;
        let key = String::from_utf8(raw_key)?;
        let value = String::from_utf8(raw_value)?;
        let key = format!("prefix_a/{key}");
        if counter >= kv_a.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_a[counter].0.clone();
        let expected_value = kv_a[counter].1.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        assert_eq!(value, expected_value, "value {} should match", counter);

        counter += 1;
    }
    assert_eq!(
        counter,
        kv_a.len(),
        "should have iterated over all prefix (prefix_a)"
    );

    let mut counter = 0;
    let query_prefix = "prefix_b".as_bytes();
    let mut range = snapshot.nonverifiable_prefix_raw(query_prefix);
    while let Some(res) = range.next().await {
        let (raw_key, raw_value) = res?;
        let key = String::from_utf8(raw_key)?;
        let value = String::from_utf8(raw_value)?;
        let key = format!("prefix_b/{key}");

        if counter >= kv_b.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_b[counter].0.clone();
        let expected_value = kv_b[counter].1.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        assert_eq!(value, expected_value, "value {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_b.len(),
        "should have iterated over all prefix (prefix_b)"
    );

    let mut counter = 0;
    let query_prefix = "key".as_bytes();
    let mut range = snapshot.nonverifiable_prefix_raw(query_prefix);
    while let Some(res) = range.next().await {
        let (raw_key, raw_value) = res?;
        let key = String::from_utf8(raw_key)?;
        let value = String::from_utf8(raw_value)?;

        tracing::debug!(?key, ?query_prefix, "iterating over prefix");

        if counter >= kv_main.len() {
            tracing::debug!(?key, ?query_prefix, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_main[counter].0.clone();
        let expected_value = kv_main[counter].1.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        assert_eq!(value, expected_value, "value {} should match", counter);
        counter += 1;
    }
    assert_eq!(
        counter,
        kv_main.len(),
        "should have iterated over all keys (main)"
    );

    Ok(())
}

#[tokio::test]
/// Test that range queries over the main store work as expected.
/// TODO(erwan): the range query implementation is broken for substores. Ignore this test for now.
#[ignore]
async fn test_substore_nv_range_queries_main_store() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let db_path = tmpdir.into_path();
    let substore_prefixes = vec!["a", "b", "c", "d"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let storage = Storage::load(db_path, substore_prefixes).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    let mut all_kv = vec![];
    let mut kv_a = vec![];
    let mut kv_c = vec![];
    let mut kv_d = vec![];
    let mut kv_main = vec![];
    for i in 0..100 {
        let key_a_i = format!("a/key_{}", i);
        let value_a_i = format!("value_{}a", i).as_bytes().to_vec();
        delta.put_raw(key_a_i.clone(), value_a_i.clone());
        all_kv.push((key_a_i.clone(), value_a_i.clone()));
        kv_a.push((key_a_i, value_a_i));
    }

    for i in 0..100 {
        let key_c_i = format!("c/key_{}", i);
        let value_c_i = format!("value_{}c", i).as_bytes().to_vec();
        delta.put_raw(key_c_i.clone(), value_c_i.clone());
        all_kv.push((key_c_i.clone(), value_c_i.clone()));
        kv_c.push((key_c_i, value_c_i));
    }

    for i in 0..100 {
        let key_i = format!("compactblock/{i:020}");
        let value_i = format!("value_{}", i).as_bytes().to_vec();
        delta.put_raw(key_i.clone(), value_i.clone());
        all_kv.push((key_i.clone(), value_i.clone()));
        kv_main.push((key_i, value_i));
    }

    for i in 0..100 {
        let key_d_i = format!("d/key_{}", i);
        let value_d_i = format!("value_{}c", i).as_bytes().to_vec();
        delta.put_raw(key_d_i.clone(), value_d_i.clone());
        all_kv.push((key_d_i.clone(), value_d_i.clone()));
        kv_d.push((key_d_i, value_d_i));
    }

    let _ = storage.commit(delta).await?;

    let snapshot = storage.latest_snapshot();

    // First, check that we can iterate over a range of compact blocks in the main store.
    // We define a range that spans all compact blocks between 12 and 34.
    let mut counter = 0;
    let start_index = 12;
    let end_index = 34;
    let mut index = start_index;

    let start_key = format!("compactblock/{:020}", start_index)
        .as_bytes()
        .to_vec();
    let end_key = format!("compactblock/{:020}", end_index)
        .as_bytes()
        .to_vec();
    let mut range = snapshot.nonverifiable_range_raw(None, start_key.clone()..end_key.clone())?;
    while let Some(res) = range.next().await {
        let (raw_key, _) = res?;
        let key = String::from_utf8(raw_key)?;
        if counter > kv_a.len() {
            tracing::debug!(?key, ?start_key, ?end_key, "unexpected key");
            panic!("prefix_keys query returned too many entries")
        }

        let expected_key = kv_main[index].0.clone();
        assert_eq!(key, expected_key, "key {} should match", counter);
        counter += 1;
        index += 1;
    }
    assert_eq!(
        counter,
        end_index - start_index,
        "should have iterated over all entries (compact block range)"
    );

    Ok(())
}
