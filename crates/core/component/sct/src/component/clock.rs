use crate::{epoch::Epoch, state_key};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use std::str::FromStr;

#[async_trait]
/// Provides read access to epoch indices, block heights, timestamps, and other related data.
pub trait EpochRead: StateRead {
    /// Get the current block height.
    ///
    /// # Errors
    /// Returns an error if the block height is missing.
    async fn get_block_height(&self) -> Result<u64> {
        self.get_proto(state_key::block_manager::block_height())
            .await?
            .ok_or_else(|| anyhow!("Missing block_height"))
    }

    /// Gets the current block timestamp from the JMT
    ///
    /// # Errors
    /// Returns an error if the block timestamp is missing.
    ///
    /// # Panic
    /// Panics if the block timestamp is not a valid RFC3339 time string.
    async fn get_current_block_timestamp(&self) -> Result<tendermint::Time> {
        let timestamp_string: String = self
            .get_proto(state_key::block_manager::current_block_timestamp())
            .await?
            .ok_or_else(|| anyhow!("Missing current_block_timestamp"))?;

        Ok(tendermint::Time::from_str(&timestamp_string)
            .context("current_block_timestamp was an invalid RFC3339 time string")?)
    }

    /// Gets a historic block timestamp from nonverifiable storage.
    ///
    /// # Errors
    /// Returns an error if the block timestamp is missing.
    ///
    /// # Panic
    /// Panics if the block timestamp is not a valid RFC3339 time string.
    async fn get_block_timestamp(&self, height: u64) -> Result<tendermint::Time> {
        let timestamp_string: String = self
            .nonverifiable_get_proto(&state_key::block_manager::block_timestamp(height).as_bytes())
            .await?
            .ok_or_else(|| anyhow!("Missing block_timestamp for height {}", height))?;

        Ok(
            tendermint::Time::from_str(&timestamp_string).context(format!(
                "block_timestamp for height {} was an invalid RFC3339 time string",
                height
            ))?,
        )
    }

    /// Get the current application epoch.
    ///
    /// # Errors
    /// Returns an error if the epoch is missing.
    async fn get_current_epoch(&self) -> Result<Epoch> {
        // Get the height
        let height = self.get_block_height().await?;

        self.get(&state_key::epoch_manager::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for current height: {height}"))
    }

    /// Get the epoch corresponding to the supplied height.
    ///
    /// # Errors
    /// Returns an error if the epoch is missing.
    async fn get_epoch_by_height(&self, height: u64) -> Result<Epoch> {
        self.get(&state_key::epoch_manager::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for height"))
    }

    /// Returns true if we are triggering an early epoch end.
    async fn is_epoch_ending_early(&self) -> bool {
        self.object_get(state_key::epoch_manager::end_epoch_early())
            .unwrap_or(false)
    }
}

impl<T: StateRead + ?Sized> EpochRead for T {}

/// Provides write access to the chain's epoch manager.
/// The epoch manager is responsible for tracking block and epoch heights
/// as well as related data like reported timestamps and epoch duration.
#[async_trait]
pub trait EpochManager: StateWrite {
    /// Writes the current block's timestamp as an RFC3339 string to verifiable storage.
    ///
    /// Also writes the current block's timestamp to the appropriate key in nonverifiable storage.
    fn put_block_timestamp(&mut self, height: u64, timestamp: tendermint::Time) {
        self.put_proto(
            state_key::block_manager::current_block_timestamp().into(),
            timestamp.to_rfc3339(),
        );

        self.nonverifiable_put_proto(
            state_key::block_manager::block_timestamp(height).into(),
            timestamp.to_rfc3339(),
        );
    }

    /// Write a value in the end epoch flag in object-storage.
    /// This is used to trigger an early epoch end at the end of the block.
    fn set_end_epoch_flag(&mut self) {
        self.object_put(state_key::epoch_manager::end_epoch_early(), true)
    }

    /// Writes the block height to verifiable storage.
    fn put_block_height(&mut self, height: u64) {
        self.put_proto(state_key::block_manager::block_height().to_string(), height)
    }

    /// Index the current epoch by height.
    fn put_epoch_by_height(&mut self, height: u64, epoch: Epoch) {
        self.put(state_key::epoch_manager::epoch_by_height(height), epoch)
    }
}

impl<T: StateWrite + ?Sized> EpochManager for T {}
