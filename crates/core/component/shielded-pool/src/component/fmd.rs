use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use decaf377_fmd::Clue;
use penumbra_sdk_proto::{
    core::component::shielded_pool::v1::{self as pb},
    StateWriteProto,
};
use penumbra_sdk_txhash::TransactionId;

use crate::fmd::state_key;

#[async_trait]
trait ClueWriteExt: StateWrite {
    fn put_current_clue_count(&mut self, count: u64) {
        self.put_raw(
            state_key::clue_count::current().to_string(),
            count.to_be_bytes().to_vec(),
        )
    }

    fn put_previous_clue_count(&mut self, count: u64) {
        self.put_raw(
            state_key::clue_count::previous().to_string(),
            count.to_be_bytes().to_vec(),
        )
    }
}

impl<T: StateWrite + ?Sized> ClueWriteExt for T {}

#[async_trait]
trait ClueReadExt: StateRead {
    // The implementation for both of these methods will return 0 on a missing key,
    // this is because the clue count is just used to tally clues over time,
    // and so 0 will always be a good starting value.
    async fn get_current_clue_count(&self) -> Result<u64> {
        Ok(self
            .get_raw(state_key::clue_count::current())
            .await?
            .map(|x| x.as_slice().try_into())
            .transpose()?
            .map(u64::from_be_bytes)
            .unwrap_or(0u64))
    }

    async fn get_previous_clue_count(&self) -> Result<u64> {
        Ok(self
            .get_raw(state_key::clue_count::previous())
            .await?
            .map(|x| x.as_slice().try_into())
            .transpose()?
            .map(u64::from_be_bytes)
            .unwrap_or(0u64))
    }
}

impl<T: StateRead + ?Sized> ClueReadExt for T {}

#[async_trait]
pub trait ClueManager: StateRead + StateWrite {
    async fn record_clue(&mut self, clue: Clue, tx: TransactionId) -> Result<()> {
        {
            let count = self.get_current_clue_count().await?;
            self.put_current_clue_count(count.saturating_add(1));
        }
        self.record_proto(pb::EventBroadcastClue {
            clue: Some(clue.into()),
            tx: Some(tx.into()),
        });
        Ok(())
    }
}

impl<T: StateRead + StateWrite> ClueManager for T {}

#[async_trait]
pub(crate) trait ClueManagerInternal: ClueManager {
    /// Flush the clue counts, returning the previous and current counts
    async fn flush_clue_count(&mut self) -> Result<(u64, u64)> {
        let previous = self.get_previous_clue_count().await?;
        let current = self.get_current_clue_count().await?;
        self.put_previous_clue_count(current);
        self.put_current_clue_count(0);
        Ok((previous, current))
    }
}

impl<T: ClueManager> ClueManagerInternal for T {}
