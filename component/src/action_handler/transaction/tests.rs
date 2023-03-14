use penumbra_chain::StateWriteExt;
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

use penumbra_transaction::Transaction;
use tendermint::Time;

use std::{sync::Arc, time::Duration};

use crate::{ActionHandler, TempStorageExt};

#[tokio::test]
async fn test_valid_before_and_after() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let now = Time::now();

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_timestamp(now);
    state_tx.apply();

    let no_timestamp = Transaction::default();
    assert!(no_timestamp.check_stateful(state.clone()).await.is_ok());

    let mut exactly_before = Transaction::default();
    exactly_before.transaction_body.valid_before = now.into();
    assert!(exactly_before.check_stateful(state.clone()).await.is_ok());

    let mut exactly_after = Transaction::default();
    exactly_after.transaction_body.valid_after = now.into();
    assert!(exactly_after.check_stateful(state.clone()).await.is_ok());

    let five_minutes_ahead = (now + Duration::from_secs(5 * 60)).unwrap();

    let mut too_early = Transaction::default();
    too_early.transaction_body.valid_after = five_minutes_ahead.into();
    assert!(too_early.check_stateful(state.clone()).await.is_err());

    let mut early_enough = Transaction::default();
    early_enough.transaction_body.valid_before = five_minutes_ahead.into();
    assert!(early_enough.check_stateful(state.clone()).await.is_ok());

    let five_minutes_ago = (now - Duration::from_secs(5 * 60)).unwrap();

    let mut late_enough = Transaction::default();
    late_enough.transaction_body.valid_after = five_minutes_ago.into();
    assert!(late_enough.check_stateful(state.clone()).await.is_ok());

    let mut too_late = Transaction::default();
    too_late.transaction_body.valid_before = five_minutes_ago.into();
    assert!(too_late.check_stateful(state.clone()).await.is_err());

    let mut just_right = Transaction::default();
    just_right.transaction_body.valid_after = five_minutes_ago.into();
    just_right.transaction_body.valid_before = five_minutes_ahead.into();
    assert!(just_right.check_stateful(state.clone()).await.is_ok());

    let mut just_wrong = Transaction::default();
    just_wrong.transaction_body.valid_after = five_minutes_ahead.into();
    just_wrong.transaction_body.valid_before = five_minutes_ago.into();
    assert!(just_wrong.check_stateful(state.clone()).await.is_err());

    Ok(())
}
