use penumbra_proto::{genesis as pb, Protobuf};
use penumbra_stake::Validator;
use serde::{Deserialize, Serialize};

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
