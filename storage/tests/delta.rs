use penumbra_storage::*;

#[tokio::test]
async fn garden_of_forking_paths() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let storage = TempStorage::new().await?;

    let mut state_init = storage.latest_state();

    // TODO: do we still want to have StateTransaction ?
    // what if we just made StateDelta be StateTransaction ?
    // what are the downsides? forced allocation for range queries?
    // where do we get the events out?
    let mut tx = state_init.begin_transaction();
    tx.put_raw("base".to_owned(), b"base".to_vec());
    tx.apply();
    storage.commit(state_init).await?;

    let mut state = storage.latest_state();
    let mut tx = state.begin_transaction();

    // We can create a StateDelta from a borrow, it will take ownership of the borrow while the family is live
    let mut delta = StateDelta::new(&mut tx);
    delta.put_raw("delta".to_owned(), b"delta".to_vec());

    // We can also nest StateDeltas -- unlike fork, this will only flatten down to the nesting point.
    let mut d2 = StateDelta::new(&mut delta);

    let mut delta_a = d2.fork();
    let mut delta_b = d2.fork();
    delta_a.put_raw("delta".to_owned(), b"delta_a".to_vec());
    delta_b.put_raw("delta".to_owned(), b"delta_b".to_vec());
    let mut delta_a_base = delta_a.fork();
    let mut delta_b_base = delta_b.fork();
    delta_a_base.delete("base".to_owned());
    delta_b_base.delete("base".to_owned());

    assert_eq!(delta_a.get_raw("base").await?, Some(b"base".to_vec()));
    assert_eq!(delta_a.get_raw("base").await?, Some(b"base".to_vec()));
    assert_eq!(delta_a_base.get_raw("base").await?, None);
    assert_eq!(delta_b_base.get_raw("base").await?, None);

    assert_eq!(delta_a.get_raw("delta").await?, Some(b"delta_a".to_vec()));
    assert_eq!(
        delta_a_base.get_raw("delta").await?,
        Some(b"delta_a".to_vec())
    );
    assert_eq!(delta_b.get_raw("delta").await?, Some(b"delta_b".to_vec()));
    assert_eq!(
        delta_b_base.get_raw("delta").await?,
        Some(b"delta_b".to_vec())
    );

    // Pick one we like and apply it, releasing the &mut delta reference...
    // Note: flattens delta_b_base -> delta_b -> delta and stops!
    delta_b_base.apply();
    // ... so we can read from delta again.
    assert_eq!(delta.get_raw("base").await?, None);
    assert_eq!(delta.get_raw("delta").await?, Some(b"delta_b".to_vec()));

    delta.apply();
    tx.apply();
    storage.commit(state).await?;

    let state = storage.latest_state();
    assert_eq!(state.get_raw("base").await?, None);
    assert_eq!(state.get_raw("delta").await?, Some(b"delta_b".to_vec()));

    Ok(())
}
