use ark_ff::Zero;
use decaf377::Fq;
use penumbra_crypto::{asset, Address, Note, Value};
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
            address: Some(
                penumbra_proto::crypto::Address::try_from(a.address)
                    .expect("able to deserialize address"),
            ),
        }
    }
}

impl TryFrom<pb::genesis_app_state::Allocation> for Allocation {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_app_state::Allocation) -> Result<Self, Self::Error> {
        Ok(Allocation {
            amount: msg.amount,
            denom: msg.denom,
            address: Address::try_from(msg.address.unwrap())?,
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
            validator: Some(
                penumbra_proto::stake::Validator::try_from(vp.validator)
                    .expect("able to deserialize validator power"),
            ),
            power: vp.power.into(),
        }
    }
}

impl TryFrom<pb::genesis_app_state::ValidatorPower> for ValidatorPower {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_app_state::ValidatorPower) -> Result<Self, Self::Error> {
        Ok(ValidatorPower {
            validator: Validator::try_from(msg.validator.unwrap())?,
            power: tendermint::vote::Power::try_from(msg.power)?,
        })
    }
}

impl Protobuf<pb::genesis_app_state::ValidatorPower> for ValidatorPower {}

/// The application state at genesis.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisAppState", into = "pb::GenesisAppState")]
pub struct AppState {
    /// The number of blocks in each epoch.
    pub epoch_duration: u64,
    /// The initial validator set.
    pub validators: Vec<ValidatorPower>,
    /// The initial token allocations.
    pub allocations: Vec<Allocation>,
}

impl From<AppState> for pb::GenesisAppState {
    fn from(a: AppState) -> Self {
        pb::GenesisAppState {
            epoch_duration: a.epoch_duration,
            // validators: a.validators.iter().map(|v| v.into()).collect(),
            validators: a
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .expect("able to serialize validators"),
            allocations: a
                .allocations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .expect("able to serialize allocations"),
        }
    }
}

impl TryFrom<pb::GenesisAppState> for AppState {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisAppState) -> Result<Self, Self::Error> {
        Ok(AppState {
            epoch_duration: msg.epoch_duration,
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
            epoch_duration: 8640,
            allocations: Vec::default(),
            validators: Vec::default(),
        }
    }
}
