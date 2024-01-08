use penumbra_chain::genesis::Content as ChainContent;
use penumbra_community_pool::genesis::Content as CommunityPoolContent;
use penumbra_distributions::genesis::Content as DistributionsContent;
use penumbra_fee::genesis::Content as FeeContent;
use penumbra_governance::genesis::Content as GovernanceContent;
use penumbra_ibc::genesis::Content as IBCContent;
use penumbra_proto::{penumbra::core::app::v1alpha1 as pb, DomainType};
use penumbra_shielded_pool::genesis::Content as ShieldedPoolContent;
use penumbra_stake::genesis::Content as StakeContent;
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

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// Stake module genesis state.
    pub stake_content: StakeContent,
    /// Shielded pool module genesis state.
    pub shielded_pool_content: ShieldedPoolContent,
    /// Governance module genesis state.
    pub governance_content: GovernanceContent,
    /// IBC module genesis state.
    pub ibc_content: IBCContent,
    /// Chain module genesis state.
    pub chain_content: ChainContent,
    /// Community Pool module genesis state.
    pub community_pool_content: CommunityPoolContent,
    /// Fee module genesis state.
    pub fee_content: FeeContent,
    /// Distributions module genesis state.
    pub distributions_content: DistributionsContent,
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
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            chain_content: Some(value.chain_content.into()),
            stake_content: Some(value.stake_content.into()),
            ibc_content: Some(value.ibc_content.into()),
            governance_content: Some(value.governance_content.into()),
            community_pool_content: Some(value.community_pool_content.into()),
            shielded_pool_content: Some(value.shielded_pool_content.into()),
            fee_content: Some(value.fee_content.into()),
            distributions_content: Some(value.distributions_content.into()),
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
            stake_content: msg
                .stake_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing stake content"))?
                .try_into()?,
            shielded_pool_content: msg
                .shielded_pool_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing shielded pool content"))?
                .try_into()?,
            governance_content: msg
                .governance_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing governance content"))?
                .try_into()?,
            ibc_content: msg
                .ibc_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing ibc content"))?
                .try_into()?,
            community_pool_content: msg
                .community_pool_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing Community Pool content"))?
                .try_into()?,
            chain_content: msg
                .chain_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing chain content"))?
                .try_into()?,
            fee_content: msg
                .fee_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing fee content"))?
                .try_into()?,
            distributions_content: msg
                .distributions_content
                .ok_or_else(|| anyhow::anyhow!("proto response missing distributions content"))?
                .try_into()?,
        })
    }
}

impl DomainType for AppState {
    type Proto = pb::GenesisAppState;
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
