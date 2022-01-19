use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{RateData, Validator, ValidatorStatus};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorInfo", into = "pb::ValidatorInfo")]
pub struct ValidatorInfo {
    pub validator: Validator,
    pub status: ValidatorStatus,
    pub rate_data: RateData,
}

impl Protobuf<pb::ValidatorInfo> for ValidatorInfo {}

impl From<ValidatorInfo> for pb::ValidatorInfo {
    fn from(v: ValidatorInfo) -> Self {
        pb::ValidatorInfo {
            validator: Some(v.validator.into()),
            status: Some(v.status.into()),
            rate_data: Some(v.rate_data.into()),
        }
    }
}

impl TryFrom<pb::ValidatorInfo> for ValidatorInfo {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorInfo) -> Result<Self, Self::Error> {
        Ok(ValidatorInfo {
            validator: v
                .validator
                .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?
                .try_into()?,
            status: v
                .status
                .ok_or_else(|| anyhow::anyhow!("missing status field in proto"))?
                .try_into()?,
            rate_data: v
                .rate_data
                .ok_or_else(|| anyhow::anyhow!("missing rate_data field in proto"))?
                .try_into()?,
        })
    }
}
