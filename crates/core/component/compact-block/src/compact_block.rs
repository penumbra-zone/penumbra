use std::{collections::BTreeMap, convert::TryFrom};

use anyhow::Result;
use penumbra_sdk_dex::{BatchSwapOutputData, TradingPair};
use penumbra_sdk_fee::GasPrices;
use penumbra_sdk_proto::{
    core::component::compact_block::v1::CompactBlockRangeResponse,
    penumbra::core::component::compact_block::v1 as pb, DomainType,
};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::fmd;
use penumbra_sdk_tct::builder::{block, epoch};
use serde::{Deserialize, Serialize};

use super::StatePayload;

/// A compressed delta update with the minimal data from a block required to
/// synchronize private client state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    /// State payloads describing new state fragments.
    pub state_payloads: Vec<StatePayload>,
    /// Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
    /// The block root of this block.
    pub block_root: block::Root,
    /// The epoch root of this epoch, if this block ends an epoch (`None` otherwise).
    pub epoch_root: Option<epoch::Root>,
    /// Latest FMD parameters. `None` if unchanged.
    pub fmd_parameters: Option<fmd::Parameters>,
    /// If the block indicated a proposal was being started.
    pub proposal_started: bool,
    /// Output prices for batch swaps occurring in this block.
    pub swap_outputs: BTreeMap<TradingPair, BatchSwapOutputData>,
    /// Set if the app parameters have been updated. Notifies the client that it should re-sync from the fullnode RPC.
    pub app_parameters_updated: bool,
    /// Updated gas prices for the native token, if they have changed.
    pub gas_prices: Option<GasPrices>,
    /// Updated gas prices for alternative fee tokens, if they have changed.
    pub alt_gas_prices: Vec<GasPrices>,
    // The epoch index
    pub epoch_index: u64,
    // **IMPORTANT NOTE FOR FUTURE HUMANS**: if you want to add new fields to the `CompactBlock`,
    // you must update `CompactBlock::requires_scanning` to check for the emptiness of those fields,
    // because the client will skip processing any compact block that is marked as not requiring
    // scanning.
}

impl Default for CompactBlock {
    fn default() -> Self {
        Self {
            height: 0,
            state_payloads: Vec::new(),
            nullifiers: Vec::new(),
            block_root: block::Finalized::default().root(),
            epoch_root: None,
            fmd_parameters: None,
            proposal_started: false,
            swap_outputs: BTreeMap::new(),
            app_parameters_updated: false,
            gas_prices: None,
            alt_gas_prices: Vec::new(),
            epoch_index: 0,
        }
    }
}

impl CompactBlock {
    /// Returns true if the compact block is empty.
    pub fn requires_scanning(&self) -> bool {
        !self.state_payloads.is_empty() // need to scan notes
            || !self.nullifiers.is_empty() // need to collect nullifiers
            || self.fmd_parameters.is_some() // need to save latest FMD parameters
            || self.proposal_started // need to process proposal start
            || self.app_parameters_updated // need to save latest app parameters
            || self.gas_prices.is_some() // need to save latest gas prices
            || !self.alt_gas_prices.is_empty() // need to save latest alt gas prices
    }
}

impl DomainType for CompactBlock {
    type Proto = pb::CompactBlock;
}

impl From<CompactBlock> for pb::CompactBlock {
    fn from(cb: CompactBlock) -> Self {
        pb::CompactBlock {
            height: cb.height,
            state_payloads: cb.state_payloads.into_iter().map(Into::into).collect(),
            nullifiers: cb.nullifiers.into_iter().map(Into::into).collect(),
            // We don't serialize block roots if they are the empty block, because we don't need to
            block_root: if cb.block_root.is_empty_finalized() {
                None
            } else {
                Some(cb.block_root.into())
            },
            epoch_root: cb.epoch_root.map(Into::into),
            fmd_parameters: cb.fmd_parameters.map(Into::into),
            proposal_started: cb.proposal_started,
            swap_outputs: cb.swap_outputs.into_values().map(Into::into).collect(),
            app_parameters_updated: cb.app_parameters_updated,
            gas_prices: cb.gas_prices.map(Into::into),
            alt_gas_prices: cb.alt_gas_prices.into_iter().map(Into::into).collect(),
            epoch_index: cb.epoch_index,
        }
    }
}

impl TryFrom<pb::CompactBlock> for CompactBlock {
    type Error = anyhow::Error;

    fn try_from(value: pb::CompactBlock) -> Result<Self, Self::Error> {
        Ok(CompactBlock {
            height: value.height,
            state_payloads: value
                .state_payloads
                .into_iter()
                .map(StatePayload::try_from)
                .collect::<Result<Vec<StatePayload>>>()?,
            swap_outputs: value
                .swap_outputs
                .into_iter()
                .map(BatchSwapOutputData::try_from)
                .map(|s| s.map(|swap_output| (swap_output.trading_pair, swap_output)))
                .collect::<Result<BTreeMap<TradingPair, BatchSwapOutputData>>>()?,
            nullifiers: value
                .nullifiers
                .into_iter()
                .map(Nullifier::try_from)
                .collect::<Result<Vec<Nullifier>>>()?,
            block_root: value
                .block_root
                .map(TryInto::try_into)
                .transpose()?
                // If the block root wasn't present, that means it's the default finalized block root
                .unwrap_or_else(|| block::Finalized::default().root()),
            epoch_root: value.epoch_root.map(TryInto::try_into).transpose()?,
            fmd_parameters: value.fmd_parameters.map(TryInto::try_into).transpose()?,
            proposal_started: value.proposal_started,
            app_parameters_updated: value.app_parameters_updated,
            gas_prices: value.gas_prices.map(TryInto::try_into).transpose()?,
            alt_gas_prices: value
                .alt_gas_prices
                .into_iter()
                .map(GasPrices::try_from)
                .collect::<Result<Vec<GasPrices>>>()?,
            epoch_index: value.epoch_index,
        })
    }
}

impl From<CompactBlock> for CompactBlockRangeResponse {
    fn from(cb: CompactBlock) -> Self {
        Self {
            compact_block: Some(cb.into()),
        }
    }
}

impl TryFrom<CompactBlockRangeResponse> for CompactBlock {
    type Error = anyhow::Error;

    fn try_from(response: CompactBlockRangeResponse) -> Result<Self, Self::Error> {
        response
            .compact_block
            .ok_or_else(|| anyhow::anyhow!("empty CompactBlockRangeResponse message"))?
            .try_into()
    }
}
