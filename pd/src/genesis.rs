use ark_ff::Zero;
use decaf377::Fq;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{asset, Address, Note, Value, CURRENT_CHAIN_ID};
use penumbra_proto::{genesis as pb, Protobuf};
use penumbra_stake::Validator;

use serde::{Deserialize, Serialize};

/// A (transparent) genesis allocation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::genesis_app_state::Allocation",
    into = "pb::genesis_app_state::Allocation"
)]
pub struct Allocation {
    pub amount: u64,
    pub denom: String,
    pub address: Address,
}

impl From<Allocation> for pb::genesis_app_state::Allocation {
    fn from(a: Allocation) -> Self {
        pb::genesis_app_state::Allocation {
            amount: a.amount,
            denom: a.denom,
            address: Some(a.address.into()),
        }
    }
}

impl TryFrom<pb::genesis_app_state::Allocation> for Allocation {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_app_state::Allocation) -> Result<Self, Self::Error> {
        Ok(Allocation {
            amount: msg.amount,
            denom: msg.denom,
            address: msg
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address field in proto"))?
                .try_into()?,
        })
    }
}

// Implement Debug manually so we can use the Display impl for the address,
// rather than dumping all the internal address components.
impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("amount", &self.amount)
            .field("denom", &self.denom)
            .field("address", &self.address.to_string())
            .finish()
    }
}

impl Allocation {
    /// Obtain a note corresponding to this allocation.
    ///
    /// Note: to ensure determinism, this uses a zero blinding factor when
    /// creating the note. This is fine, because the genesis allocations are
    /// already public.
    pub fn note(&self) -> Result<Note, anyhow::Error> {
        Note::from_parts(
            *self.address.diversifier(),
            *self.address.transmission_key(),
            Value {
                amount: self.amount,
                asset_id: asset::REGISTRY
                    .parse_denom(&self.denom)
                    .ok_or_else(|| anyhow::anyhow!("invalid denomination"))?
                    .id(),
            },
            Fq::zero(),
        )
        .map_err(Into::into)
    }
}

impl Protobuf<pb::genesis_app_state::Allocation> for Allocation {}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(
    try_from = "pb::genesis_app_state::ValidatorPower",
    into = "pb::genesis_app_state::ValidatorPower"
)]
pub struct ValidatorPower {
    pub validator: Validator,
    pub power: tendermint::vote::Power,
}

impl From<ValidatorPower> for pb::genesis_app_state::ValidatorPower {
    fn from(vp: ValidatorPower) -> Self {
        pb::genesis_app_state::ValidatorPower {
            validator: Some(vp.validator.into()),
            power: vp.power.into(),
        }
    }
}

impl TryFrom<pb::genesis_app_state::ValidatorPower> for ValidatorPower {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_app_state::ValidatorPower) -> Result<Self, Self::Error> {
        Ok(ValidatorPower {
            validator: msg
                .validator
                .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?
                .try_into()?,
            power: msg.power.try_into()?,
        })
    }
}

impl Protobuf<pb::genesis_app_state::ValidatorPower> for ValidatorPower {}

/// The application state at genesis.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisAppState", into = "pb::GenesisAppState")]
pub struct AppState {
    /// Global configuration for the chain, such as chain ID and epoch duration.
    pub chain_params: ChainParams,
    /// The initial validator set.
    pub validators: Vec<ValidatorPower>,
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
            chain_params: msg.chain_params.unwrap().into(),
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

impl Default for AppState {
    fn default() -> Self {
        AppState {
            chain_params: ChainParams {
                chain_id: CURRENT_CHAIN_ID.to_string(),
                epoch_duration: 10,
                unbonding_epochs: 5,
            },
            allocations: Vec::default(),
            validators: Vec::default(),
        }
    }
}
