use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::connection::ChainId;
use penumbra_proto::{StateReadProto, StateWriteProto};
use tendermint::Time;

use crate::{
    params::{ChainParameters, FmdParameters},
    state_key, Epoch,
};

/// This trait provides read access to chain-related parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the chain parameters have been updated in this block.
    fn chain_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::chain_params_updated())
            .is_some()
    }

    /// Gets the app chain parameters from the JMT.
    async fn get_chain_params(&self) -> Result<ChainParameters> {
        self.get(state_key::chain_params())
            .await?
            .ok_or_else(|| anyhow!("Missing ChainParameters"))
    }

    /// Gets the current epoch for the chain.
    async fn get_current_epoch(&self) -> Result<Epoch> {
        // Get the height
        let height = self.get_block_height().await?;

        self.get(&state_key::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for current height"))
    }

    /// Gets the epoch duration for the chain.
    async fn get_epoch_duration(&self) -> Result<u64> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        self.get_chain_params()
            .await
            .map(|params| params.epoch_duration)
    }

    async fn get_epoch_for_height(&self, height: u64) -> Result<Epoch> {
        self.get(&state_key::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for height"))
    }

    /// Gets the chain ID.
    async fn get_chain_id(&self) -> Result<String> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        self.get_chain_params().await.map(|params| params.chain_id)
    }

    /// Gets the chain revision number, from the chain ID
    async fn get_revision_number(&self) -> Result<u64> {
        let cid_str = self.get_chain_id().await?;

        Ok(ChainId::from_string(&cid_str).version())
    }

    /// Gets the current block height from the JMT
    async fn get_block_height(&self) -> Result<u64> {
        let height_bytes: u64 = self
            .get_proto(state_key::block_height())
            .await?
            .ok_or_else(|| anyhow!("Missing block_height"))?;

        Ok(height_bytes)
    }

    /// Gets the current block timestamp from the JMT
    async fn get_block_timestamp(&self) -> Result<Time> {
        let timestamp_string: String = self
            .get_proto(state_key::block_timestamp())
            .await?
            .ok_or_else(|| anyhow!("Missing block_timestamp"))?;

        Ok(Time::from_str(&timestamp_string)
            .context("block_timestamp was an invalid RFC3339 time string")?)
    }

    /// Checks a provided chain_id against the chain state.
    ///
    /// Passes through if the provided chain_id is empty or matches, and
    /// otherwise errors.
    async fn check_chain_id(&self, provided: &str) -> Result<()> {
        let chain_id = self
            .get_chain_id()
            .await
            .context(format!("error getting chain id: '{provided}'"))?;
        if provided.is_empty() || provided == chain_id {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "provided chain_id {} does not match chain_id {}",
                provided,
                chain_id
            ))
        }
    }

    /// Gets the current FMD parameters from the JMT.
    async fn get_current_fmd_parameters(&self) -> Result<FmdParameters> {
        self.get(state_key::fmd_parameters_current())
            .await?
            .ok_or_else(|| anyhow!("Missing FmdParameters"))
    }

    /// Gets the previous FMD parameters from the JMT.
    async fn get_previous_fmd_parameters(&self) -> Result<FmdParameters> {
        self.get(state_key::fmd_parameters_previous())
            .await?
            .ok_or_else(|| anyhow!("Missing FmdParameters"))
    }

    /// Get the current epoch.
    async fn epoch(&self) -> Result<Epoch> {
        // Get the height
        let height = self.get_block_height().await?;

        self.get(&state_key::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for current height: {height}"))
    }

    /// Returns true if the chain is halted (or will be halted momentarily).
    async fn is_chain_halted(&self, total_halt_count: u64) -> Result<bool> {
        Ok(self
            .nonverifiable_get_raw(&state_key::halted(total_halt_count))
            .await?
            .is_some())
    }

    /// Returns true if the next height is an upgrade height.
    /// We look-ahead to the next height because we want to halt the chain immediately after
    /// committing the block.
    async fn is_upgrade_height(&self) -> Result<bool> {
        let Some(next_upgrade_height) = self
            .nonverifiable_get_raw(state_key::next_upgrade().as_bytes())
            .await?
        else {
            return Ok(false);
        };

        let next_upgrade_height = u64::from_be_bytes(next_upgrade_height.as_slice().try_into()?);

        let current_height = self.get_block_height().await?;
        Ok(current_height.saturating_add(1) == next_upgrade_height)
    }

    async fn epoch_by_height(&self, height: u64) -> Result<Epoch> {
        self.get(&state_key::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for height"))
    }

    // Returns true if the epoch is ending early this block.
    fn epoch_ending_early(&self) -> bool {
        self.object_get(state_key::end_epoch_early())
            .unwrap_or(false)
    }

    /// Get the current chain halt count.
    async fn chain_halt_count(&self) -> Result<u64> {
        Ok(self
            .get_proto::<u64>(state_key::chain_halt_count())
            .await?
            .unwrap_or_default())
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// This trait provides write access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided chain parameters to the JMT.
    fn put_chain_params(&mut self, params: ChainParameters) {
        // Note that the chain parameters have been updated:
        self.object_put(state_key::chain_params_updated(), ());

        // Change the chain parameters:
        self.put(state_key::chain_params().into(), params)
    }

    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) {
        self.put_proto(state_key::block_height().to_string(), height)
    }

    /// Writes the epoch for the current height
    fn put_epoch_by_height(&mut self, height: u64, epoch: Epoch) {
        self.put(state_key::epoch_by_height(height), epoch)
    }

    /// Writes the block timestamp to the JMT
    fn put_block_timestamp(&mut self, timestamp: Time) {
        self.put_proto(state_key::block_timestamp().into(), timestamp.to_rfc3339())
    }

    /// Writes the current FMD parameters to the JMT.
    fn put_current_fmd_parameters(&mut self, params: FmdParameters) {
        self.put(state_key::fmd_parameters_current().into(), params)
    }

    /// Writes the previous FMD parameters to the JMT.
    fn put_previous_fmd_parameters(&mut self, params: FmdParameters) {
        self.put(state_key::fmd_parameters_previous().into(), params)
    }

    /// Signals to the consensus worker to halt after the next commit.
    async fn signal_halt(&mut self) -> Result<()> {
        let halt_count = self.chain_halt_count().await?;

        // Increment the current halt count unconditionally...
        self.put_proto(state_key::chain_halt_count().to_string(), halt_count + 1);

        // ...and signal that a halt should occur if the halt count is fresh (`is_chain_halted` will
        // check against the total number of expected chain halts to determine whether a halt should
        // actually occur).
        self.nonverifiable_put_raw(state_key::halted(halt_count).to_vec(), vec![]);

        Ok(())
    }

    /// Record the next upgrade height.
    /// Right after committing the state for this height, the chain will halt and wait for an upgrade.
    /// It uses the same mechanism as emergency halting to prevent the chain from restarting.
    async fn signal_upgrade(&mut self, height: u64) -> Result<()> {
        self.nonverifiable_put_raw(
            state_key::next_upgrade().into(),
            height.to_be_bytes().to_vec(),
        );
        Ok(())
    }

    // Signals that the epoch should end this block.
    fn signal_end_epoch(&mut self) {
        self.object_put(state_key::end_epoch_early(), true)
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
