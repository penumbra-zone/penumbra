use std::{collections::BTreeMap, convert::TryFrom};

use anyhow::Result;
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_crypto::Nullifier;
use penumbra_dex::{BatchSwapOutputData, TradingPair};
use penumbra_proto::{
    client::v1alpha1::CompactBlockRangeResponse, core::chain::v1alpha1 as pb, DomainType, TypeUrl,
};
use penumbra_tct::builder::{block, epoch};
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
    pub fmd_parameters: Option<FmdParameters>,
    /// If the block indicated a proposal was being started.
    pub proposal_started: bool,
    /// Output prices for batch swaps occurring in this block.
    pub swap_outputs: BTreeMap<TradingPair, BatchSwapOutputData>,
    /// Updated chain parameters, if any have changed.
    pub chain_parameters: Option<ChainParameters>,
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
            chain_parameters: None,
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
            || self.chain_parameters.is_some() // need to save latest chain parameters
    }
}

impl TypeUrl for CompactBlock {
    const TYPE_URL: &'static str = "/penumbra.core.chain.v1alpha1.CompactBlock";
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
            chain_parameters: cb.chain_parameters.map(Into::into),
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
            chain_parameters: value.chain_parameters.map(TryInto::try_into).transpose()?,
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
