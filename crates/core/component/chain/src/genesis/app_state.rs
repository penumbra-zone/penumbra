use anyhow::Context;
use penumbra_proto::{
    core::chain::v1alpha1 as pb, core::stake::v1alpha1 as pb_stake, DomainType, TypeUrl,
};
use serde::{Deserialize, Serialize};

use super::Allocation;
use crate::params::ChainParameters;

/// The application state at genesis.
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
    /// Global configuration for the chain, such as chain ID and epoch duration.
    pub chain_params: ChainParameters,
    /// The initial validator set.
    pub validators: Vec<pb_stake::Validator>,
    /// The initial token allocations.
    pub allocations: Vec<Allocation>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Content(Default::default())
    }
}

impl Default for Content {
    fn default() -> Self {
        Self {
            chain_params: Default::default(),
            // TODO: create a test validator
            validators: Default::default(),
            allocations: vec![
                Allocation {
                    amount: 1000u128.into(),
                    denom: "penumbra"
                        .parse()
                        .expect("hardcoded \"penumbra\" denom should be parseable"),
                    address: crate::test_keys::ADDRESS_0_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    amount: 100u128.into(),
                    denom: "test_usd"
                        .parse()
                        .expect("hardcoded \"test_usd\" denom should be parseable"),
                    address: crate::test_keys::ADDRESS_0_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    amount: 100u128.into(),
                    denom: "gm"
                        .parse()
                        .expect("hardcoded \"gm\" denom should be parseable"),
                    address: crate::test_keys::ADDRESS_1_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    amount: 100u128.into(),
                    denom: "gn"
                        .parse()
                        .expect("hardcoded \"gn\" denom should be parseable"),
                    address: crate::test_keys::ADDRESS_1_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
            ],
        }
    }
}

impl From<AppState> for pb::GenesisAppState {
    fn from(a: AppState) -> Self {
        let genesis_state = match a {
            AppState::Content(c) => {
                pb::genesis_app_state::GenesisAppState::GenesisContent(c.into())
            }
            AppState::Checkpoint(h) => {
                pb::genesis_app_state::GenesisAppState::GenesisCheckpoint(h.into())
            }
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
            validators: value.validators.into_iter().map(Into::into).collect(),
            allocations: value.allocations.into_iter().map(Into::into).collect(),
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
                Ok(AppState::Checkpoint(h.try_into()?))
            }
        }
    }
}

impl TryFrom<pb::GenesisCheckpoint> for AppHash {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisCheckpoint) -> Result<Self, Self::Error> {
        let msg = msg.app_hash;
        if msg.len() != 32 {
            return Err(anyhow::anyhow!(
                "app hash must be 32 bytes, got {}",
                msg.len()
            ));
        }

        let h: [u8; 32] = msg.try_into().expect("array is 32 bytes");
        Ok(AppHash(h))
    }
}

impl From<AppHash> for pb::GenesisCheckpoint {
    fn from(h: AppHash) -> Self {
        pb::GenesisCheckpoint {
            app_hash: h.0.into(),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            chain_params: msg
                .chain_params
                .context("chain params not present in protobuf message")?
                .try_into()?,
            validators: msg
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,

            allocations: msg
                .allocations
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
        assert!(a.validators.len() == 0);
        Ok(())
    }
}
