use penumbra_proto::{core::chain::v1alpha1 as pb, core::stake::v1alpha1 as pb_stake, Protobuf};
use serde::{Deserialize, Serialize};

use super::Allocation;
use crate::params::ChainParameters;

/// The application state at genesis.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(try_from = "pb::GenesisAppState", into = "pb::GenesisAppState")]
pub struct AppState {
    /// Global configuration for the chain, such as chain ID and epoch duration.
    pub chain_params: ChainParameters,
    /// The initial validator set.
    pub validators: Vec<pb_stake::Validator>,
    /// The initial token allocations.
    pub allocations: Vec<Allocation>,
}

impl From<AppState> for pb::GenesisAppState {
    fn from(a: AppState) -> Self {
        pb::GenesisAppState {
            validators: a.validators.into_iter().map(Into::into).collect(),
            allocations: a.allocations.into_iter().map(Into::into).collect(),
            chain_params: Some(a.chain_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisAppState> for AppState {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisAppState) -> Result<Self, Self::Error> {
        Ok(AppState {
            chain_params: msg.chain_params.unwrap().try_into()?,
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

impl Protobuf<pb::GenesisAppState> for AppState {}
