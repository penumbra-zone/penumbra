use std::str::FromStr;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_storage2::{StateRead, StateWrite};
use tendermint::Time;

use crate::{
    params::{ChainParameters, FmdParameters},
    state_key, Epoch,
};

/// This trait provides read access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the chain parameters from the JMT.
    async fn get_chain_params(&self) -> Result<ChainParameters> {
        self.get(state_key::chain_params())
            .await?
            .ok_or_else(|| anyhow!("Missing ChainParameters"))
    }

    /// Gets the current epoch for the chain.
    async fn get_current_epoch(&self) -> Result<Epoch> {
        let block_height = self.get_block_height().await?;
        Ok(Epoch::from_height(
            block_height,
            self.get_epoch_duration().await?,
        ))
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

    /// Gets the chain ID.
    async fn get_chain_id(&self) -> Result<String> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        self.get_chain_params().await.map(|params| params.chain_id)
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

        Ok(Time::from_str(&timestamp_string).unwrap())
    }

    /// Checks a provided chain_id against the chain state.
    ///
    /// Passes through if the provided chain_id is empty or matches, and
    /// otherwise errors.
    async fn check_chain_id(&self, provided: &str) -> Result<(), tonic::Status> {
        let chain_id = self
            .get_chain_id()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting chain id: {}", e)))?;
        if provided.is_empty() || provided == chain_id {
            Ok(())
        } else {
            Err(tonic::Status::failed_precondition(format!(
                "provided chain_id {} does not match chain_id {}",
                provided, chain_id
            )))
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

        // Get the epoch duration
        let epoch_duration = self.get_epoch_duration().await?;

        // The current epoch
        Ok(Epoch::from_height(height, epoch_duration))
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
        self.put(state_key::chain_params().into(), params)
    }

    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) {
        self.put_proto("block_height".into(), height)
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
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
