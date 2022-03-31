use std::str::FromStr;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_chain::params::ChainParams;
use tendermint::Time;

use crate::WriteOverlayExt;

/// This trait provides read and write access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
#[async_trait]
pub trait PenumbraStore: WriteOverlayExt {
    /// Gets the chain parameters from the JMT.
    async fn get_chain_params(&self) -> Result<ChainParams> {
        self.get_domain(b"chain_params".into())
            .await?
            .ok_or_else(|| anyhow!("Missing ChainParams"))
    }

    /// Writes the provided chain parameters to the JMT.
    async fn put_chain_params(&self, params: ChainParams) {
        self.put_domain(b"chain_params".into(), params).await
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

    /// Gets the current block height from the JMT
    async fn get_block_height(&self) -> Result<u64> {
        let height_bytes: u64 = self
            .get_proto(b"block_height".into())
            .await?
            .ok_or_else(|| anyhow!("Missing block_height"))?;

        Ok(height_bytes)
    }

    /// Writes the block height to the JMT
    async fn put_block_height(&self, height: u64) {
        self.put_proto(b"block_height".into(), height).await
    }

    /// Gets the current block timestamp from the JMT
    async fn get_block_timestamp(&self) -> Result<Time> {
        let timestamp_string: String = self
            .get_proto(b"block_timestamp".into())
            .await?
            .ok_or_else(|| anyhow!("Missing block_timestamp"))?;

        Ok(Time::from_str(&timestamp_string).unwrap())
    }

    /// Writes the block timestamp to the JMT
    async fn put_block_timestamp(&self, timestamp: Time) {
        self.put_proto(b"block_timestamp".into(), timestamp.to_rfc3339())
            .await
    }
}

impl<T: WriteOverlayExt> PenumbraStore for T {}
