use std::cmp::Ordering;

use penumbra_crypto::{asset, stake::Penalty, Amount};
use penumbra_proto::client::v1alpha1 as pb_client;
use penumbra_proto::core::chain::v1alpha1 as pb_chain;
use penumbra_proto::core::crypto::v1alpha1 as pb_crypto;
use penumbra_proto::view::v1alpha1 as pb_view;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

pub mod change;

#[derive(Clone, Debug)]
pub struct AssetInfo {
    pub asset_id: asset::Id,
    pub denom: asset::Denom,
    pub as_of_block_height: u64,
    pub total_supply: u64,
}

impl DomainType for AssetInfo {
    type Proto = pb_chain::AssetInfo;
}

impl TryFrom<pb_chain::AssetInfo> for AssetInfo {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::AssetInfo) -> Result<Self, Self::Error> {
        Ok(AssetInfo {
            asset_id: asset::Id::try_from(msg.asset_id.unwrap())?,
            denom: asset::Denom::try_from(msg.denom.unwrap())?,
            as_of_block_height: msg.as_of_block_height,
            total_supply: msg.total_supply,
        })
    }
}

impl From<AssetInfo> for pb_chain::AssetInfo {
    fn from(ai: AssetInfo) -> Self {
        pb_chain::AssetInfo {
            asset_id: Some(pb_crypto::AssetId::from(ai.asset_id)),
            denom: Some(pb_crypto::Denom::from(ai.denom)),
            as_of_block_height: ai.as_of_block_height,
            total_supply: ai.total_supply,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb_chain::ChainParameters",
    into = "pb_chain::ChainParameters"
)]
pub struct ChainParameters {
    pub chain_id: String,
    pub epoch_duration: u64,

    pub unbonding_epochs: u64,
    /// The number of validators allowed in the consensus set (Active state).
    pub active_validator_limit: u64,
    /// The base reward rate, expressed in basis points of basis points
    pub base_reward_rate: u64,
    /// The penalty for slashing due to misbehavior, expressed in basis points squared (10^-8)
    pub slashing_penalty_misbehavior: Penalty,
    /// The penalty for slashing due to downtime, expressed in basis points squared (10^-8)
    pub slashing_penalty_downtime: Penalty,
    /// The number of blocks in the window to check for downtime.
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    pub missed_blocks_maximum: u64,

    /// Whether IBC (forming connections, processing IBC packets) is enabled.
    pub ibc_enabled: bool,
    /// Whether inbound ICS-20 transfers are enabled
    pub inbound_ics20_transfers_enabled: bool,
    /// Whether outbound ICS-20 transfers are enabled
    pub outbound_ics20_transfers_enabled: bool,

    /// The number of blocks during which a proposal is voted on.
    pub proposal_voting_blocks: u64,
    /// The deposit required to create a proposal.
    pub proposal_deposit_amount: Amount,
    /// The quorum required for a proposal to be considered valid, as a fraction of the total stake
    /// weight of the network.
    pub proposal_valid_quorum: Ratio,
    /// The threshold for a proposal to pass voting, as a ratio of "yes" votes over "no" votes.
    pub proposal_pass_threshold: Ratio,
    /// The threshold for a proposal to be vetoed, as a ratio of "no" votes over all total votes.
    pub proposal_veto_threshold: Ratio,
}

impl DomainType for ChainParameters {
    type Proto = pb_chain::ChainParameters;
}

impl TryFrom<pb_chain::ChainParameters> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::ChainParameters) -> anyhow::Result<Self> {
        Ok(ChainParameters {
            chain_id: msg.chain_id,
            epoch_duration: msg.epoch_duration,
            unbonding_epochs: msg.unbonding_epochs,
            active_validator_limit: msg.active_validator_limit,
            slashing_penalty_downtime: msg
                .slashing_penalty_downtime
                .ok_or_else(|| {
                    anyhow::anyhow!("slashing_penalty_downtime_bps must be set in ChainParameters")
                })?
                .try_into()?,
            slashing_penalty_misbehavior: msg
                .slashing_penalty_misbehavior
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "slashing_penalty_misbehavior_bps must be set in ChainParameters"
                    )
                })?
                .try_into()?,
            base_reward_rate: msg.base_reward_rate,
            missed_blocks_maximum: msg.missed_blocks_maximum,
            signed_blocks_window_len: msg.signed_blocks_window_len,
            ibc_enabled: msg.ibc_enabled,
            inbound_ics20_transfers_enabled: msg.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: msg.outbound_ics20_transfers_enabled,
            proposal_voting_blocks: msg.proposal_voting_blocks,
            proposal_deposit_amount: msg
                .proposal_deposit_amount
                .ok_or_else(|| {
                    anyhow::anyhow!("proposal_deposit_amount must be set in ChainParameters")
                })?
                .try_into()?,
            proposal_valid_quorum: msg
                .proposal_valid_quorum
                .ok_or_else(|| anyhow::anyhow!("missing `proposal_valid_quorum`"))?
                .into(),
            proposal_pass_threshold: msg
                .proposal_pass_threshold
                .ok_or_else(|| anyhow::anyhow!("missing `proposal_pass_threshold`"))?
                .into(),
            proposal_veto_threshold: msg
                .proposal_veto_threshold
                .ok_or_else(|| anyhow::anyhow!("missing `proposal_veto_threshold`"))?
                .into(),
        })
    }
}

impl TryFrom<pb_view::ChainParametersResponse> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb_view::ChainParametersResponse) -> Result<Self, Self::Error> {
        response
            .parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?
            .try_into()
    }
}

impl TryFrom<pb_client::ChainParametersResponse> for ChainParameters {
    type Error = anyhow::Error;

    fn try_from(response: pb_client::ChainParametersResponse) -> Result<Self, Self::Error> {
        response
            .chain_parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?
            .try_into()
    }
}

impl From<ChainParameters> for pb_chain::ChainParameters {
    fn from(params: ChainParameters) -> Self {
        pb_chain::ChainParameters {
            chain_id: params.chain_id,
            epoch_duration: params.epoch_duration,
            unbonding_epochs: params.unbonding_epochs,
            active_validator_limit: params.active_validator_limit,
            signed_blocks_window_len: params.signed_blocks_window_len,
            missed_blocks_maximum: params.missed_blocks_maximum,
            slashing_penalty_downtime: Some(params.slashing_penalty_downtime.into()),
            slashing_penalty_misbehavior: Some(params.slashing_penalty_misbehavior.into()),
            base_reward_rate: params.base_reward_rate,
            ibc_enabled: params.ibc_enabled,
            inbound_ics20_transfers_enabled: params.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: params.outbound_ics20_transfers_enabled,
            proposal_voting_blocks: params.proposal_voting_blocks,
            proposal_deposit_amount: Some(params.proposal_deposit_amount.into()),
            proposal_valid_quorum: Some(params.proposal_valid_quorum.into()),
            proposal_pass_threshold: Some(params.proposal_pass_threshold.into()),
            proposal_veto_threshold: Some(params.proposal_veto_threshold.into()),
        }
    }
}

// TODO: defaults are implemented here as well as in the
// `pd::main`
impl Default for ChainParameters {
    fn default() -> Self {
        Self {
            chain_id: String::new(),
            epoch_duration: 719,
            unbonding_epochs: 2,
            active_validator_limit: 80,
            // copied from cosmos hub
            signed_blocks_window_len: 10000,
            missed_blocks_maximum: 9500,
            // 1000 basis points = 10%
            slashing_penalty_misbehavior: Penalty(1000_0000),
            // 1 basis point = 0.01%
            slashing_penalty_downtime: Penalty(1_0000),
            // 3bps -> 11% return over 365 epochs
            base_reward_rate: 3_0000,
            ibc_enabled: true,
            inbound_ics20_transfers_enabled: false,
            outbound_ics20_transfers_enabled: false,
            // governance
            proposal_voting_blocks: 720,
            proposal_deposit_amount: 10_000_000u64.into(), // 10,000,000 upenumbra = 10 penumbra
            // governance parameters copied from cosmos hub
            proposal_valid_quorum: Ratio::new(2, 5),
            proposal_pass_threshold: Ratio::new(1, 2),
            // veto threshold means if (no / no + yes + abstain) > veto_threshold, then proposal is vetoed
            proposal_veto_threshold: Ratio::new(4, 5),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::FmdParameters", into = "pb_chain::FmdParameters")]
pub struct FmdParameters {
    /// Bits of precision.
    pub precision_bits: u8,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl DomainType for FmdParameters {
    type Proto = pb_chain::FmdParameters;
}

impl TryFrom<pb_chain::FmdParameters> for FmdParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb_chain::FmdParameters) -> Result<Self, Self::Error> {
        Ok(FmdParameters {
            precision_bits: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<FmdParameters> for pb_chain::FmdParameters {
    fn from(params: FmdParameters) -> Self {
        pb_chain::FmdParameters {
            precision_bits: u32::from(params.precision_bits),
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for FmdParameters {
    fn default() -> Self {
        Self {
            precision_bits: 0,
            as_of_block_height: 1,
        }
    }
}

/// This is a ratio of two `u64` values, intended to be used solely in governance parameters and
/// tallying. It only implements construction and comparison, not arithmetic, to reduce the trusted
/// codebase for governance.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::Ratio", into = "pb_chain::Ratio")]
pub struct Ratio {
    numerator: u64,
    denominator: u64,
}

impl Ratio {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        // Convert everything to `u128` to avoid overflow when multiplying
        u128::from(self.numerator) * u128::from(other.denominator)
            == u128::from(self.denominator) * u128::from(other.numerator)
    }
}

impl Eq for Ratio {}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        // Convert everything to `u128` to avoid overflow when multiplying
        (u128::from(self.numerator) * u128::from(other.denominator))
            .cmp(&(u128::from(self.denominator) * u128::from(other.numerator)))
    }
}

impl From<Ratio> for pb_chain::Ratio {
    fn from(ratio: Ratio) -> Self {
        pb_chain::Ratio {
            numerator: ratio.numerator,
            denominator: ratio.denominator,
        }
    }
}

impl From<pb_chain::Ratio> for Ratio {
    fn from(msg: pb_chain::Ratio) -> Self {
        Ratio {
            numerator: msg.numerator,
            denominator: msg.denominator,
        }
    }
}
