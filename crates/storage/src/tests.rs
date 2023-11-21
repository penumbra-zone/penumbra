use crate::*;
use futures::StreamExt;

/// Checks that deleting a nonexistent key behaves as expected (no errors, it's already gone)
#[tokio::test]
async fn delete_nonexistent_key() -> anyhow::Result<()> {
    let tmpdir = tempfile::tempdir()?;
    // Initialize an empty Storage in the new directory
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let mut state_init = StateDelta::new(storage.latest_snapshot());
    state_init.delete("nonexist".to_string());
    storage.commit(state_init).await?;

    Ok(())
}

#[tokio::test]
/// In rare cases, the database lock has not been (yet) released by the time
/// the next Storage::load() call is made. This is fixed by `Storage::release()`
/// which mimicks the behavior of an async drop (releasing resources).
async fn db_lock_is_released() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    storage.release().await;

    Ok(())
}

#[tokio::test]
async fn simple_flow() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    // Initialize an empty Storage in the new directory
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    // Version -1 to Version 0 writes
    //
    // tx00: test => test
    // tx00: c/aa => 0 [object store]
    // tx00: c/ab => 1 [object store]
    // tx00: c/ac => 2 [object store]
    // tx00: c/ad => 3 [object store]
    // tx00: iA => A [nonverifiable store]
    // tx00: iC => C [nonverifiable store]
    // tx00: iF => F [nonverifiable store]
    // tx00: iD => D [nonverifiable store]
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
    // tx10: iB => B [nonverifiable store]
    // tx11: a/ab => ab2
    // tx11: iD => [deleted] nonverifiable store]

    let mut state_init = StateDelta::new(storage.latest_snapshot());
    // Check that reads on an empty state return Ok(None)
    assert_eq!(state_init.get_raw("test").await?, None);
    assert_eq!(state_init.get_raw("a/aa").await?, None);

    // Create tx00
    let mut tx00 = StateDelta::new(&mut state_init);
    tx00.put_raw("test".to_owned(), b"test".to_vec());
    tx00.object_put("c/aa", 0u64);
    tx00.object_put("c/ab", 1u64);
    tx00.object_put("c/ac", 2u64);
    tx00.object_put("c/ad", 3u64);
    tx00.nonverifiable_put_raw(b"iA".to_vec(), b"A".to_vec());
    tx00.nonverifiable_put_raw(b"iC".to_vec(), b"C".to_vec());
    tx00.nonverifiable_put_raw(b"iF".to_vec(), b"F".to_vec());
    tx00.nonverifiable_put_raw(b"iD".to_vec(), b"D".to_vec());

    // Check reads against tx00:
    //     This is present in tx00
    assert_eq!(tx00.get_raw("test").await?, Some(b"test".to_vec()));
    //     This is missing in tx00 and state_init and tree is empty
    assert_eq!(tx00.get_raw("a/aa").await?, None);
    //     Present in tx00 object store
    assert_eq!(tx00.object_get("c/aa"), Some(0u64));
    assert_eq!(tx00.object_get("c/ab"), Some(1u64));
    assert_eq!(tx00.object_get("c/ac"), Some(2u64));
    assert_eq!(tx00.object_get("c/ad"), Some(3u64));
    //     Missing in tx00 object store
    assert_eq!(tx00.object_get::<bool>("nonexist"), None);
    //     Nonconsensus range checks
    let mut range = tx00.nonverifiable_prefix_raw(b"i");
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
    assert_eq!(state_init.object_get("c/aa"), Some(0u64));
    assert_eq!(state_init.object_get("c/ab"), Some(1u64));
    assert_eq!(state_init.object_get("c/ac"), Some(2u64));
    assert_eq!(state_init.object_get("c/ad"), Some(3u64));
    //     Missing in state_init object store
    assert_eq!(state_init.object_get::<bool>("nonexist"), None);
    //     Nonconsensus range checks
    let mut range = state_init.nonverifiable_prefix_raw(b"i");
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
    let mut tx01 = StateDelta::new(&mut state_init);
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
    let mut range_keys = tx01.prefix_keys("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/aa".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/aaa".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/ab".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range_keys.next().await.transpose()?, Some("a/z".to_owned()));
    assert_eq!(range.next().await.transpose()?, None);
    assert_eq!(range_keys.next().await.transpose()?, None);
    std::mem::drop(range);
    std::mem::drop(range_keys);

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
    let mut range_keys = state_init.prefix_keys("a/");
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aa".to_owned(), b"aa".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/aa".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/aaa".to_owned(), b"aaa".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/aaa".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/ab".to_owned(), b"ab".to_vec()))
    );
    assert_eq!(
        range_keys.next().await.transpose()?,
        Some("a/ab".to_owned())
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some(("a/z".to_owned(), b"z".to_vec()))
    );
    assert_eq!(range_keys.next().await.transpose()?, Some("a/z".to_owned()));
    assert_eq!(range.next().await.transpose()?, None);
    assert_eq!(range_keys.next().await.transpose()?, None);
    std::mem::drop(range);
    std::mem::drop(range_keys);

    // Now commit state_init to storage
    storage.commit(state_init).await?;

    // Now we have version 0.
    let state0 = storage.latest_snapshot();
    assert_eq!(state0.version(), 0);
    let mut state0 = StateDelta::new(state0);

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
    //     Nonconsensus prefix checks
    let mut range = state0.nonverifiable_prefix_raw(b"i");
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
    let mut tx10 = StateDelta::new(&mut state0);
    tx10.delete("test".to_owned());
    tx10.delete("a/aaa".to_owned());
    tx10.put_raw("a/c".to_owned(), b"c".to_vec());
    tx10.nonverifiable_put_raw(b"iB".to_vec(), b"B".to_vec());

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
    //     Nonconsensus prefix checks
    let mut range = tx10.nonverifiable_prefix_raw(b"i");
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
    let mut tx11 = StateDelta::new(&mut state0);
    tx11.put_raw("a/ab".to_owned(), b"ab2".to_vec());
    tx11.nonverifiable_delete(b"iD".to_vec());

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
    let mut range = tx11.nonverifiable_prefix_raw(b"i");
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
    let mut range = state0.nonverifiable_prefix_raw(b"i");
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
    let state0a = storage.latest_snapshot();
    assert_eq!(state0a.version(), 0);

    // Commit state0 as state1.
    storage.commit(state0).await?;

    let state1 = storage.latest_snapshot();
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
    let mut range = state1.nonverifiable_prefix_raw(b"i");
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
    let mut range = state0a.nonverifiable_prefix_raw(b"i");
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
    storage.release().await;
    // std::mem::drop(state0); // consumed in commit()
    std::mem::drop(state0a);
    std::mem::drop(state1);

    // Now reload the storage from the same directory...
    let storage_a = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    let state1a = storage_a.latest_snapshot();

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
    let mut range = state1a.nonverifiable_prefix_raw(b"i");
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
    let mut range = state1a.nonverifiable_range_raw(Some(b"i"), b"A".to_vec()..b"C".to_vec())?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    let mut range = state1a.nonverifiable_range_raw(Some(b"i"), b"B".to_vec()..b"C".to_vec())?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iB".to_vec(), b"B".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = state1a.nonverifiable_range_raw(Some(b"i"), b"A".to_vec()..b"F".to_vec())?;
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
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = state1a.nonverifiable_range_raw(Some(b"i"), b"A".to_vec()..)?;
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

    let mut range = state1a.nonverifiable_range_raw(Some(b"i"), ..)?;
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

    let mut range = state1a.nonverifiable_range_raw(None, b"i".to_vec()..)?;
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

    let mut range = state1a.nonverifiable_range_raw(None, b"iA".to_vec()..b"iB".to_vec())?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iA".to_vec(), b"A".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    Ok(())
}

#[tokio::test]
/// Test that range queries over the nonverifiable store work as expected.
/// A range query can have a prefix, a start key, and an end key, so we want to test:
/// - queries with no prefix, no start key, and no end key
/// - queries with no prefix, a start key, and no end key
/// - queries with no prefix, no start key, and an end key
/// - queries with no prefix, a start key, and an end key
/// - queries with a prefix, no start key, and no end key
/// - queries with a prefix, a start key, and no end key
/// - queries with a prefix, no start key, and an end key
async fn range_queries_basic() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;
    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let mut state_init = StateDelta::new(storage.latest_snapshot());

    // Check that range queries over an empty store work does not return any results.
    let mut range = state_init.nonverifiable_range_raw(None, ..)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut tx00 = StateDelta::new(&mut state_init);
    tx00.nonverifiable_put_raw(b"iA".to_vec(), b"A".to_vec());
    tx00.nonverifiable_put_raw(b"iC".to_vec(), b"C".to_vec());
    tx00.nonverifiable_put_raw(b"iF".to_vec(), b"F".to_vec());
    tx00.nonverifiable_put_raw(
        b"random_key_not_prefixed".to_vec(),
        b"quetzalcoatl".to_vec(),
    );

    // Check that keys with the wrong prefix are not included in the results.
    let mut range = tx00.nonverifiable_range_raw(Some(b"i"), ..)?;
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
        Some((b"iF".to_vec(), b"F".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Insert an entry that precedes the current last entry
    tx00.nonverifiable_put_raw(b"iD".to_vec(), b"D".to_vec());

    // Check that the new entry is included in the results.
    let mut range = tx00.nonverifiable_range_raw(Some(b"i"), ..)?;
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

    tx00.apply();
    // Check that the new entry is included in the results.
    let mut range = state_init.nonverifiable_range_raw(Some(b"i"), ..)?;
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

    let mut tx01 = StateDelta::new(&mut state_init);
    tx01.nonverifiable_delete(b"iA".to_vec());
    tx01.nonverifiable_put_raw(b"iC".to_vec(), b"China".to_vec());
    tx01.nonverifiable_put_raw(b"iD".to_vec(), b"Denmark".to_vec());
    tx01.nonverifiable_put_raw(b"iF".to_vec(), b"Finland".to_vec());

    // Check that the updated entries are included in the results.
    let mut range = tx01.nonverifiable_range_raw(Some(b"i"), ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"China".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"Denmark".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"Finland".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check that setting the lower bound to the first key doesn't change the results.
    let mut range = tx01.nonverifiable_range_raw(Some(b"i"), b"C".to_vec()..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"China".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"Denmark".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"Finland".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check that setting the upper bound to a key that doesn't exist doesn't change the results.
    let mut range = tx01.nonverifiable_range_raw(Some(b"i"), ..b"Z".to_vec())?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"China".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"Denmark".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"Finland".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check that using a manually prefixed keys doesn't change the results.
    let mut range = tx01.nonverifiable_range_raw(None, b"i".to_vec()..b"iZ".to_vec())?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"China".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"Denmark".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"Finland".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    tx01.apply();

    // Check that all entries are included in the results.
    let mut range = state_init.nonverifiable_range_raw(None, ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iC".to_vec(), b"China".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iD".to_vec(), b"Denmark".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"iF".to_vec(), b"Finland".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((
            b"random_key_not_prefixed".to_vec(),
            b"quetzalcoatl".to_vec()
        ))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut tx02 = StateDelta::new(&mut state_init);

    for i in 0..=100 {
        tx02.nonverifiable_put_raw(
            format!("compact_block/{:020}", i).as_bytes().to_vec(),
            format!("{}", i).as_bytes().to_vec(),
        );
    }

    // Check that all compact blocks are included in the results.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), ..)?;
    for i in 0..=100 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            ))
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    let cb_10 = format!("{:020}", 10).as_bytes().to_vec();

    // Check that setting a lower bound works.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_10.clone()..)?;
    for i in 10..=100 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            )),
            "i={}",
            i
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let cb_20 = format!("{:020}", 20).as_bytes().to_vec();

    // Check that specifying a full range works.
    let mut range =
        tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_10.clone()..cb_20.clone())?;
    for i in 10..20 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            ))
        );
    }
    assert_eq!(range.next().await.transpose()?, None);

    // Check that leaving the lower bound unspecified works.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), ..cb_20)?;
    for i in 0..20 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            ))
        );
    }
    assert_eq!(range.next().await.transpose()?, None);

    // Delete compact blocks [9;21]
    let deleted_keys = (9..=21)
        .map(|i| format!("compact_block/{:020}", i).as_bytes().to_vec())
        .collect::<Vec<_>>();
    for key in deleted_keys.iter() {
        tx02.nonverifiable_delete(key.clone());
    }

    // Check that the deleted compact blocks are not included in the results.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), ..)?;
    for i in 0..=100 {
        if (9..=21).contains(&i) {
            continue;
        }
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            )),
            "i={}",
            i
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let cb_9 = format!("{:020}", 9).as_bytes().to_vec();
    let cb_15 = format!("{:020}", 15).as_bytes().to_vec();
    let cb_21 = format!("{:020}", 21).as_bytes().to_vec();
    let cb_22 = format!("{:020}", 22).as_bytes().to_vec();
    let cb_23 = format!("{:020}", 23).as_bytes().to_vec();

    // Check that the deleted compact blocks are not included in the results, even if they're in the bound argument.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_9.clone()..)?;
    for i in 22..=100 {
        let found = range.next().await.transpose()?;
        let foundstr = String::from_utf8(found.clone().unwrap().0).unwrap();
        println!("{i}: foundstr={}", foundstr);
        assert_eq!(
            found,
            Some((
                format!("compact_block/{:020}", i).as_bytes().to_vec(),
                format!("{}", i).as_bytes().to_vec()
            ))
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check a variety of deleted bounds.
    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_9.clone()..cb_15)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_9.clone()..cb_21)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_9.clone()..cb_22)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = tx02.nonverifiable_range_raw(Some(b"compact_block/"), cb_9.clone()..cb_23)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((
            format!("compact_block/{:020}", 22).as_bytes().to_vec(),
            format!("{}", 22).as_bytes().to_vec()
        ))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    Ok(())
}

#[tokio::test]
/// Test that overwrites work correctly, and that we can read the latest value.
async fn range_query_overwrites() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let state_init = StateDelta::new(storage.latest_snapshot());
    // Check that reads on an empty state return Ok(None)
    let mut range = state_init.nonverifiable_range_raw(None, ..)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    Ok(())
}

#[tokio::test]
/// Test that inserting a value that precedes the peeked value works.
async fn range_query_prepend_peeked_value() -> anyhow::Result<()> {
    use crate::StateWrite;
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let mut state_init = StateDelta::new(storage.latest_snapshot());
    state_init.nonverifiable_put_raw(b"b".to_vec(), b"beluga".to_vec());
    state_init.nonverifiable_put_raw(b"c".to_vec(), b"charm".to_vec());
    storage.commit(state_init).await?;

    let mut state = StateDelta::new(storage.latest_snapshot());
    let mut range = state.nonverifiable_range_raw(None, ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b".to_vec(), b"beluga".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"c".to_vec(), b"charm".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    state.nonverifiable_put_raw(b"a".to_vec(), b"aroma".to_vec());

    // Check that the new value preceding the first peeked value is returned (no bound)
    let mut range = state.nonverifiable_range_raw(None, ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a".to_vec(), b"aroma".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b".to_vec(), b"beluga".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"c".to_vec(), b"charm".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check that the new value preceding the first peeked value is returned (with bound)
    let mut range = state.nonverifiable_range_raw(None, b"a".to_vec()..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a".to_vec(), b"aroma".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b".to_vec(), b"beluga".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"c".to_vec(), b"charm".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    // Check that the new value preceding the first peeked value is NOT returned.
    let mut range = state.nonverifiable_range_raw(None, b"b".to_vec()..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b".to_vec(), b"beluga".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"c".to_vec(), b"charm".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    Ok(())
}

#[tokio::test]
/// Test that specifying an inverted range does not work.
async fn range_query_ordering() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let mut state_init = StateDelta::new(storage.latest_snapshot());
    let mut range = state_init.nonverifiable_range_raw(None, ..)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    state_init.nonverifiable_put_raw(b"c/charm".to_vec(), b"charm".to_vec());
    state_init.nonverifiable_put_raw(b"a/aroma".to_vec(), b"aroma".to_vec());
    state_init.nonverifiable_put_raw(b"a/apple".to_vec(), b"apple".to_vec());
    state_init.nonverifiable_put_raw(b"b/boat".to_vec(), b"boat".to_vec());

    let mut range = state_init.nonverifiable_range_raw(None, ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/apple".to_vec(), b"apple".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aroma".to_vec(), b"aroma".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b/boat".to_vec(), b"boat".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"c/charm".to_vec(), b"charm".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let lower = b"b/".to_vec();
    let upper = b"c/".to_vec();
    let mut range = state_init.nonverifiable_range_raw(None, lower..upper)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b/boat".to_vec(), b"boat".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let upper1 = b"c/".to_vec();
    let upper2 = b"c/".to_vec();
    let mut range = state_init.nonverifiable_range_raw(None, upper1..upper2)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let lower = b"b/".to_vec();
    let upper = b"c/".to_vec();
    let range = state_init.nonverifiable_range_raw(None, upper..lower);
    assert!(range.is_err());
    std::mem::drop(range);

    Ok(())
}

#[tokio::test]
/// Test that passing in an absurd range does not work.
async fn range_query_bad_range() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;

    let state_init = StateDelta::new(storage.latest_snapshot());

    let lower = format!("{:020}", 0).as_bytes().to_vec();
    let upper = format!("{:020}", 0).as_bytes().to_vec();

    // Inclusive range are not supported.
    let range = state_init.nonverifiable_range_raw(None, lower..=upper);
    assert!(range.is_err());
    std::mem::drop(range);

    let lower = format!("{:020}", 0).as_bytes().to_vec();
    let upper = format!("{:020}", 1).as_bytes().to_vec();
    // Inclusive range are not supported.
    let range = state_init.nonverifiable_range_raw(None, upper..lower);
    assert!(range.is_err());

    Ok(())
}

#[tokio::test]
/// Test that the semantics of the range query do not change when the state is
/// persisted.
async fn range_query_storage_basic() -> anyhow::Result<()> {
    use crate::read::StateRead;
    use crate::write::StateWrite;
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    for height in 0..100 {
        delta.nonverifiable_put_raw(
            format!("compact_block/{:020}", height).as_bytes().to_vec(),
            format!("compact_block/{:020}", height).as_bytes().to_vec(),
        );
    }

    // We insert keys before the compact block keys.
    for key in 0..10 {
        delta.nonverifiable_put_raw(
            format!("ante/{:020}", key).as_bytes().to_vec(),
            format!("ante/{:020}", key).as_bytes().to_vec(),
        );
    }

    // We insert keys after the compact block keys.
    for key in 0..10 {
        delta.nonverifiable_put_raw(
            format!("post/{:020}", key).as_bytes().to_vec(),
            format!("postƒ/{:020}", key).as_bytes().to_vec(),
        );
    }

    storage.commit(delta).await?;

    let state_init = storage.latest_snapshot();
    let mut range = state_init.nonverifiable_range_raw(Some(b"compact_block/"), ..)?;
    for height in 0..100 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", height).as_bytes().to_vec(),
                format!("compact_block/{:020}", height).as_bytes().to_vec()
            )),
            "height: {}",
            height
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let cb_50 = format!("{:020}", 50).as_bytes().to_vec();
    let cb_80 = format!("{:020}", 80).as_bytes().to_vec();
    let mut range = state_init.nonverifiable_range_raw(Some(b"compact_block/"), cb_50..cb_80)?;
    for height in 50..80 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", height).as_bytes().to_vec(),
                format!("compact_block/{:020}", height).as_bytes().to_vec()
            )),
            "height: {}",
            height
        );
    }
    assert_eq!(range.next().await.transpose()?, None);

    let cb_80 = format!("{:020}", 80).as_bytes().to_vec();
    let mut range = state_init.nonverifiable_range_raw(Some(b"compact_block/"), ..cb_80)?;
    for height in 0..80 {
        assert_eq!(
            range.next().await.transpose()?,
            Some((
                format!("compact_block/{:020}", height).as_bytes().to_vec(),
                format!("compact_block/{:020}", height).as_bytes().to_vec()
            )),
            "height: {}",
            height
        );
    }
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    Ok(())
}

#[tokio::test]
/// Test that prefixed range queries work over the persisted state.
async fn range_query_storage() -> anyhow::Result<()> {
    use crate::read::StateRead;
    use crate::write::StateWrite;
    let _ = tracing_subscriber::fmt::try_init();
    let tmpdir = tempfile::tempdir()?;

    let storage = Storage::load(tmpdir.path().to_owned(), vec![]).await?;
    let mut delta = StateDelta::new(storage.latest_snapshot());

    delta.nonverifiable_put_raw(b"a/aaaaa".to_vec(), b"1".to_vec());
    delta.nonverifiable_put_raw(b"a/aaaab".to_vec(), b"2".to_vec());
    delta.nonverifiable_put_raw(b"a/aaaac".to_vec(), b"3".to_vec());
    delta.nonverifiable_put_raw(b"b/boat".to_vec(), b"4".to_vec());
    storage.commit(delta).await?;

    let state_init = storage.latest_snapshot();
    let mut range = state_init.nonverifiable_range_raw(Some(b"a/"), ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaaa".to_vec(), b"1".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaab".to_vec(), b"2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaac".to_vec(), b"3".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = state_init.nonverifiable_range_raw(Some(b"a/"), b"aaaa".to_vec()..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaaa".to_vec(), b"1".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaab".to_vec(), b"2".to_vec()))
    );
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"a/aaaac".to_vec(), b"3".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = state_init.nonverifiable_range_raw(Some(b"b/"), b"chorizo".to_vec()..)?;
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);

    let mut range = state_init.nonverifiable_range_raw(Some(b"b/"), ..)?;
    assert_eq!(
        range.next().await.transpose()?,
        Some((b"b/boat".to_vec(), b"4".to_vec()))
    );
    assert_eq!(range.next().await.transpose()?, None);
    std::mem::drop(range);
    Ok(())
}
