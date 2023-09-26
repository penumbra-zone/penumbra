use anyhow::Context;
use penumbra_proto::{
    // TODO: avoid this import!
    core::component::stake::v1alpha1 as pb_stake,
    penumbra::core::component::chain::v1alpha1 as pb,
    DomainType,
    TypeUrl,
};
use serde::{Deserialize, Serialize};

use super::Allocation;
use penumbra_app::params::AppParameters;

/// The application state at genesis.
/// TODO: bubble up to penumbra_app (https://github.com/penumbra-zone/penumbra/issues/3085)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisAppState", into = "pb::GenesisAppState")]
pub enum AppState {
    /// The application state at genesis.
    Content(Content),
    /// The checkpointed application state at genesis, contains a free-form hash.
    Checkpoint(Vec<u8>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// Global configuration for the app.
    pub app_params: AppParameters,
    /// Stake module genesis state.
    pub stake_content: StakeContent,
    /// Shielded pool module genesis state.
    pub shielded_pool_content: ShieldedPoolContent,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Content(Default::default())
    }
}

impl Default for Content {
    fn default() -> Self {
        Self {
            app_params: Default::default(),
            stake_content: Default::default(),
            shielded_pool_content: Default::default(),
        }
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
            chain_params: Some(value.chain_params.into()),
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
            app_params: msg
                .app_params
                .context("app params not present in protobuf message")?
                .try_into()?,
            stake_content: msg
                .stake_content
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            shielded_pool_content: msg
                .shielded_pool_content
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TypeUrl for AppState {
    const TYPE_URL: &'static str = "/penumbra.core.chain.v1alpha1.GenesisAppState";
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
        assert!(a.validators.is_empty());
        Ok(())
    }
}
