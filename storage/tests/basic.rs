use futures::StreamExt;
use penumbra_storage::*;

#[tokio::test]
async fn simple_flow() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let tmpdir = tempfile::tempdir()?;

    // Initialize an empty Storage in the new directory
    let storage = Storage::load(tmpdir.path().to_owned()).await?;

    // Version -1 to Version 0 writes
    //
    // tx00: test => test
    // tx00: c/aa => 0 [object store]
    // tx00: c/ab => 1 [object store]
    // tx00: c/ac => 2 [object store]
    // tx00: c/ad => 3 [object store]
    // tx00: iA => A [nonconsensus store]
    // tx00: iC => C [nonconsensus store]
    // tx00: iF => F [nonconsensus store]
    // tx00: iD => D [nonconsensus store]
    // tx01: a/aa => aa
    // tx01: a/aaa => aaa
    // tx01: a/ab => ab
    // tx01: a/z  => z
    // tx01: c/ab => 10 [object store]
    // tx01: c/ac => [deleted] [object store]
    //
    // Version 0 to Version 1 writes
    // tx10: test => [deleted]
    // tx10: a/aaa => [deleted]
    // tx10: a/c => c
    // tx10: iB => B [nonconsensus store]
    // tx11: a/ab => ab2
    // tx11: iD => [deleted] nonconsensus store]

    let mut state_init = storage.latest_state();
    // Check that reads on an empty state return Ok(None)
    assert_eq!(state_init.get_raw("test").await?, None);
    assert_eq!(state_init.get_raw("a/aa").await?, None);

    // Create tx00
    let mut tx00 = state_init.begin_transaction();
    tx00.put_raw("test".to_owned(), b"test".to_vec());
    tx00.object_put("c/aa", 0u64);
    tx00.object_put("c/ab", 1u64);
    tx00.object_put("c/ac", 2u64);
    tx00.object_put("c/ad", 3u64);
    tx00.nonconsensus_put_raw(b"iA".to_vec(), b"A".to_vec());
    tx00.nonconsensus_put_raw(b"iC".to_vec(), b"C".to_vec());
    tx00.nonconsensus_put_raw(b"iF".to_vec(), b"F".to_vec());
    tx00.nonconsensus_put_raw(b"iD".to_vec(), b"D".to_vec());

    // Check reads against tx00:
    //     This is present in tx00
    assert_eq!(tx00.get_raw("test").await?, Some(b"test".to_vec()));
    //     This is missing in tx00 and state_init and tree is empty
    assert_eq!(tx00.get_raw("a/aa").await?, None);
    //     Present in tx00 object store
    assert_eq!(tx00.object_get("c/aa"), Some(&0u64));
    assert_eq!(tx00.object_get("c/ab"), Some(&1u64));
    assert_eq!(tx00.object_get("c/ac"), Some(&2u64));
    assert_eq!(tx00.object_get("c/ad"), Some(&3u64));
    //     Present in tx00 object store but requested with wrong type
    assert_eq!(tx00.object_get::<bool>("c/aa"), None);
    //     Missing in tx00 object store
    assert_eq!(tx00.object_get::<bool>("nonexist"), None);
    //     Nonconsensus range checks
    let mut range = tx00.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"D".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Now apply the transaction to state_init
    tx00.apply();
    assert_eq!(state_init.get_raw("test").await?, Some(b"test".to_vec()));
    assert_eq!(state_init.get_raw("a/aa").await?, None);
    //     Present in state_init object store
    assert_eq!(state_init.object_get("c/aa"), Some(&0u64));
    assert_eq!(state_init.object_get("c/ab"), Some(&1u64));
    assert_eq!(state_init.object_get("c/ac"), Some(&2u64));
    assert_eq!(state_init.object_get("c/ad"), Some(&3u64));
    //     Present in state_init object store but requested with wrong type
    assert_eq!(state_init.object_get::<bool>("c/aa"), None);
    //     Missing in state_init object store
    assert_eq!(state_init.object_get::<bool>("nonexist"), None);
    //     Nonconsensus range checks
    let mut range = state_init.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"D".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Create a transaction writing the other keys.
    let mut tx01 = state_init.begin_transaction();
    tx01.put_raw("a/aa".to_owned(), b"aa".to_vec());
    tx01.put_raw("a/aaa".to_owned(), b"aaa".to_vec());
    tx01.put_raw("a/ab".to_owned(), b"ab".to_vec());
    tx01.put_raw("a/z".to_owned(), b"z".to_vec());
    tx01.object_put("c/ab", 10u64);
    tx01.object_delete("c/ac");

    // Check reads against tx01:
    //    This is missing in tx01 and reads through to state_init
    assert_eq!(tx01.get_raw("test").await?, Some(b"test".to_vec()));
    //    This is present in tx01
    assert_eq!(tx01.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(tx01.get_raw("a/aaa").await?, Some(b"aaa".to_vec()));
    assert_eq!(tx01.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(tx01.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is missing in tx01 and in state_init
    assert_eq!(tx01.get_raw("a/c").await?, None);
    let mut range = tx01.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Now apply the transaction to state_init
    tx01.apply();

    // Check reads against state_init:
    //    This is present in state_init
    assert_eq!(state_init.get_raw("test").await?, Some(b"test".to_vec()));
    assert_eq!(state_init.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state_init.get_raw("a/aaa").await?, Some(b"aaa".to_vec()));
    assert_eq!(state_init.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(state_init.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is missing in state_init
    assert_eq!(state_init.get_raw("a/c").await?, None);
    let mut range = state_init.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Now commit state_init to storage
    storage.commit(state_init).await?;

    // Now we have version 0.
    let mut state0 = storage.latest_state();
    assert_eq!(state0.version(), 0);
    // Check reads against state0:
    //    This is missing in state0 and present in JMT
    assert_eq!(state0.get_raw("test").await?, Some(b"test".to_vec()));
    assert_eq!(state0.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state0.get_raw("a/aaa").await?, Some(b"aaa".to_vec()));
    assert_eq!(state0.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(state0.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is missing in state0 and missing in JMT
    assert_eq!(state0.get_raw("a/c").await?, None);
    let mut range = state0.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    //     Nonconsensus range checks
    let mut range = state0.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"D".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Start building a transaction
    let mut tx10 = state0.begin_transaction();
    tx10.delete("test".to_owned());
    tx10.delete("a/aaa".to_owned());
    tx10.put_raw("a/c".to_owned(), b"c".to_vec());
    tx10.nonconsensus_put_raw(b"iB".to_vec(), b"B".to_vec());

    // Check reads against tx10:
    //    This is deleted in tx10, missing in state0, present in JMT
    assert_eq!(tx10.get_raw("test").await?, None);
    assert_eq!(tx10.get_raw("a/aaa").await?, None);
    //    This is missing in tx10, missing in state0, present in JMT
    assert_eq!(tx10.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(tx10.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(tx10.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is present in tx10, missing in state0, missing in JMT
    assert_eq!(tx10.get_raw("a/c").await?, Some(b"c".to_vec()));
    let mut range = tx10.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    //     Nonconsensus range checks
    let mut range = tx10.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"D".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Apply tx10 to state0
    tx10.apply();

    // Check reads against state0
    //    This is deleted in state0, present in JMT
    assert_eq!(state0.get_raw("test").await?, None);
    assert_eq!(state0.get_raw("a/aaa").await?, None);
    //    This is missing in state0, present in JMT
    assert_eq!(state0.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state0.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(state0.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is present in state0, missing in JMT
    assert_eq!(state0.get_raw("a/c").await?, Some(b"c".to_vec()));
    let mut range = state0.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Start building another transaction
    let mut tx11 = state0.begin_transaction();
    tx11.put_raw("a/ab".to_owned(), b"ab2".to_vec());
    tx11.nonconsensus_delete(b"iD".to_vec());

    // Check reads against tx11:
    //    This is present in tx11, missing in state0, present in JMT
    assert_eq!(tx11.get_raw("a/ab").await?, Some(b"ab2".to_vec()));
    //    This is missing in tx11, deleted in state0, present in JMT
    assert_eq!(tx11.get_raw("test").await?, None);
    assert_eq!(tx11.get_raw("a/aaa").await?, None);
    //    This is missing in tx11, missing in state0, present in JMT
    assert_eq!(tx11.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(tx11.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is missing in tx10, present in state0, missing in JMT
    assert_eq!(tx11.get_raw("a/c").await?, Some(b"c".to_vec()));
    let mut range = tx11.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    //     Nonconsensus range checks
    let mut range = tx11.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Apply tx11 to state0
    tx11.apply();

    // Check reads against state0
    //    This is deleted in state0, present in JMT
    assert_eq!(state0.get_raw("test").await?, None);
    assert_eq!(state0.get_raw("a/aaa").await?, None);
    //    This is missing in state0, present in JMT
    assert_eq!(state0.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state0.get_raw("a/z").await?, Some(b"z".to_vec()));
    //    This is present in state0, missing in JMT
    assert_eq!(state0.get_raw("a/c").await?, Some(b"c".to_vec()));
    //    This is present in state0, present in JMT
    assert_eq!(state0.get_raw("a/ab").await?, Some(b"ab2".to_vec()));
    let mut range = state0.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    let mut range = state0.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Create another fork of state 0 while we've edited the first one but before we commit.
    let state0a = storage.latest_state();
    assert_eq!(state0a.version(), 0);

    // Commit state0 as state1.
    storage.commit(state0).await?;

    let state1 = storage.latest_state();
    assert_eq!(state1.version(), 1);

    // Check reads against state1
    assert_eq!(state1.get_raw("test").await?, None);
    assert_eq!(state1.get_raw("a/aaa").await?, None);
    assert_eq!(state1.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state1.get_raw("a/ab").await?, Some(b"ab2".to_vec()));
    assert_eq!(state1.get_raw("a/z").await?, Some(b"z".to_vec()));
    assert_eq!(state1.get_raw("a/c").await?, Some(b"c".to_vec()));
    let mut range = state1.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    let mut range = state1.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check reads against state0a
    assert_eq!(state0a.get_raw("test").await?, Some(b"test".to_vec()));
    assert_eq!(state0a.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state0a.get_raw("a/aaa").await?, Some(b"aaa".to_vec()));
    assert_eq!(state0a.get_raw("a/ab").await?, Some(b"ab".to_vec()));
    assert_eq!(state0a.get_raw("a/z").await?, Some(b"z".to_vec()));
    assert_eq!(state0a.get_raw("a/c").await?, None);
    let mut range = state0a.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    //     Nonconsensus range checks
    let mut range = state0a.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"D".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Now, check that closing and reloading works.

    // First, be sure to explicitly drop anything keeping a reference to the
    // RocksDB instance:
    std::mem::drop(storage);
    // std::mem::drop(state0); // consumed in commit()
    std::mem::drop(state0a);
    std::mem::drop(state1);

    // Now reload the storage from the same directory...
    let storage_a = Storage::load(tmpdir.path().to_owned()).await?;
    let state1a = storage_a.latest_state();

    // Check that we reload at the correct version ...
    assert_eq!(state1a.version(), 1);

    // Check reads against state1a after reloading the DB
    assert_eq!(state1a.get_raw("test").await?, None);
    assert_eq!(state1a.get_raw("a/aaa").await?, None);
    assert_eq!(state1a.get_raw("a/aa").await?, Some(b"aa".to_vec()));
    assert_eq!(state1a.get_raw("a/ab").await?, Some(b"ab2".to_vec()));
    assert_eq!(state1a.get_raw("a/z").await?, Some(b"z".to_vec()));
    assert_eq!(state1a.get_raw("a/c").await?, Some(b"c".to_vec()));
    let mut range = state1a.prefix_raw("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/c".to_owned(), b"c".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    //     Nonconsensus range checks
    let mut range = state1a.nonconsensus_prefix_raw(b"i");
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"C".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    Ok(())
}
