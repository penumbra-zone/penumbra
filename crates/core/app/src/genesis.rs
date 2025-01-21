use penumbra_sdk_auction::genesis::Content as AuctionContent;
use penumbra_sdk_community_pool::genesis::Content as CommunityPoolContent;
use penumbra_sdk_dex::genesis::Content as DexContent;
use penumbra_sdk_distributions::genesis::Content as DistributionsContent;
use penumbra_sdk_fee::genesis::Content as FeeContent;
use penumbra_sdk_funding::genesis::Content as FundingContent;
use penumbra_sdk_governance::genesis::Content as GovernanceContent;
use penumbra_sdk_ibc::genesis::Content as IBCContent;
use penumbra_sdk_proto::{penumbra::core::app::v1 as pb, DomainType};
use penumbra_sdk_sct::genesis::Content as SctContent;
use penumbra_sdk_shielded_pool::genesis::Content as ShieldedPoolContent;
use penumbra_sdk_stake::genesis::Content as StakeContent;
use serde::{Deserialize, Serialize};

/// The application state at genesis.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisAppState", into = "pb::GenesisAppState")]
#[allow(clippy::large_enum_variant)]
pub enum AppState {
    /// The application state at genesis.
    Content(Content),
    /// The checkpointed application state at genesis, contains a free-form hash.
    Checkpoint(Vec<u8>),
}

impl AppState {
    pub fn content(&self) -> Option<&Content> {
        match self {
            AppState::Content(content) => Some(content),
            _ => None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The chain ID.
    pub chain_id: String,
    /// Community Pool module genesis state.
    pub community_pool_content: CommunityPoolContent,
    /// Distributions module genesis state.
    pub distributions_content: DistributionsContent,
    /// Fee module genesis state.
    pub fee_content: FeeContent,
    /// Funding module genesis state.
    pub funding_content: FundingContent,
    /// Governance module genesis state.
    pub governance_content: GovernanceContent,
    /// IBC module genesis state.
    pub ibc_content: IBCContent,
    // Sct module genesis state.
    pub sct_content: SctContent,
    /// Shielded pool module genesis state.
    pub shielded_pool_content: ShieldedPoolContent,
    /// Stake module genesis state.
    pub stake_content: StakeContent,
    /// Dex component genesis state.
    pub dex_content: DexContent,
    /// Auction component genesis state.
    pub auction_content: AuctionContent,
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl Default for AppState {
    fn default() -> Self {
        Self::Content(Default::default())
    }
}

impl From<AppState> for pb::GenesisAppState {
    fn from(a: AppState) -> Self {
        let genesis_state = match a {
            AppState::Content(c) => {
                pb::genesis_app_state::GenesisAppState::GenesisContent(c.into())
            }
            AppState::Checkpoint(h) => pb::genesis_app_state::GenesisAppState::GenesisCheckpoint(h),
        };

        pb::GenesisAppState {
            genesis_app_state: Some(genesis_state),
        }
    }
}

impl From<Content> for pb::GenesisContent {
    fn from(genesis: Content) -> Self {
        pb::GenesisContent {
            chain_id: genesis.chain_id,
            auction_content: Some(genesis.auction_content.into()),
            community_pool_content: Some(genesis.community_pool_content.into()),
            distributions_content: Some(genesis.distributions_content.into()),
            fee_content: Some(genesis.fee_content.into()),
            funding_content: Some(genesis.funding_content.into()),
            governance_content: Some(genesis.governance_content.into()),
            ibc_content: Some(genesis.ibc_content.into()),
            sct_content: Some(genesis.sct_content.into()),
            shielded_pool_content: Some(genesis.shielded_pool_content.into()),
            stake_content: Some(genesis.stake_content.into()),
            dex_content: Some(genesis.dex_content.into()),
        }
    }
}

impl TryFrom<pb::GenesisAppState> for AppState {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisAppState) -> Result<Self, Self::Error> {
        let state = msg
            .genesis_app_state
            .ok_or_else(|| anyhow::anyhow!("missing genesis_app_state field in proto"))?;
        match state {
            pb::genesis_app_state::GenesisAppState::GenesisContent(c) => {
                Ok(AppState::Content(c.try_into()?))
            }
            pb::genesis_app_state::GenesisAppState::GenesisCheckpoint(h) => {
                Ok(AppState::Checkpoint(h))
            }
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            chain_id: msg.chain_id,
            auction_content: msg
                .auction_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing Auction content"))?
                .try_into()?,
            community_pool_content: msg
                .community_pool_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing Community Pool content"))?
                .try_into()?,
            distributions_content: msg
                .distributions_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing distributions content"))?
                .try_into()?,
            governance_content: msg
                .governance_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing governance content"))?
                .try_into()?,
            fee_content: msg
                .fee_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing fee content"))?
                .try_into()?,
            funding_content: msg
                .funding_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing funding content"))?
                .try_into()?,
            ibc_content: msg
                .ibc_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing ibc content"))?
                .try_into()?,
            sct_content: msg
                .sct_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing sct content"))?
                .try_into()?,
            shielded_pool_content: msg
                .shielded_pool_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing shielded pool content"))?
                .try_into()?,
            stake_content: msg
                .stake_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing stake content"))?
                .try_into()?,
            dex_content: msg
                .dex_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing dex content"))?
                .try_into()?,
        })
    }
}

impl DomainType for AppState {
    type Proto = pb::GenesisAppState;
}

impl Content {
    pub fn with_chain_id(self, chain_id: String) -> Self {
        Self { chain_id, ..self }
    }

    pub fn with_epoch_duration(self, epoch_duration: u64) -> Self {
        Self {
            sct_content: penumbra_sdk_sct::genesis::Content {
                sct_params: penumbra_sdk_sct::params::SctParameters { epoch_duration },
            },
            ..self
        }
    }

    pub fn with_unbonding_delay(self, unbonding_delay: u64) -> Self {
        Self {
            stake_content: penumbra_sdk_stake::genesis::Content {
                stake_params: penumbra_sdk_stake::params::StakeParameters {
                    unbonding_delay,
                    ..self.stake_content.stake_params
                },
                ..self.stake_content
            },
            ..self
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that the default implementation of contains zero validators,
    /// requiring validators to be passed in out of band. N.B. there's also a
    /// `validators` field in the [`tendermint::Genesis`] struct, which we don't use,
    /// preferring the AppState definition instead.
    #[test]
    fn check_validator_defaults() -> anyhow::Result<()> {
        let a = Content {
            ..Default::default()
        };
        assert!(a.stake_content.validators.is_empty());
        Ok(())
    }
}
