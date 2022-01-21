use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{RateData, Validator, ValidatorStatus};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorInfo", into = "pb::ValidatorInfo")]
pub struct ValidatorInfo {
    validator: Validator,
    status: ValidatorStatus,
    rate_data: RateData,
}

impl ValidatorInfo {
    pub fn new(
        validator: Validator,
        status: ValidatorStatus,
        rate_data: RateData,
    ) -> anyhow::Result<Self> {
        if validator.identity_key != status.identity_key
            || validator.identity_key != rate_data.identity_key
        {
            return Err(anyhow::anyhow!(
                "validator, status, and rate data identity keys must match"
            ));
        }

        Ok(Self {
            validator,
            status,
            rate_data,
        })
    }

    /// Get the validator.
    pub fn validator(&self) -> &Validator {
        &self.validator
    }

    /// Get the status of this validator.
    pub fn status(&self) -> &ValidatorStatus {
        &self.status
    }

    /// Get the rate data for this validator.
    pub fn rate_data(&self) -> &RateData {
        &self.rate_data
    }

    /// Extract the components of this struct, consuming `self`.
    pub fn into_parts(self) -> (Validator, ValidatorStatus, RateData) {
        (self.validator, self.status, self.rate_data)
    }
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
