use std::str::FromStr;

use anyhow::Context;
use penumbra_sdk_proto::{core::component::governance::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// An encoded parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::EncodedParameter", into = "pb::EncodedParameter")]
pub struct EncodedParameter {
    pub component: String,
    pub key: String,
    pub value: String,
}

impl DomainType for EncodedParameter {
    type Proto = pb::EncodedParameter;
}

impl TryFrom<pb::EncodedParameter> for EncodedParameter {
    type Error = anyhow::Error;
    fn try_from(value: pb::EncodedParameter) -> Result<Self, Self::Error> {
        // TODO: what are max key/value lengths here?
        // Validation:
        // - Key has max length of 64 chars
        if value.key.len() > 64 {
            anyhow::bail!("key length must be less than or equal to 64 characters");
        }

        // - Value has max length of 2048 chars
        if value.value.len() > 2048 {
            anyhow::bail!("value length must be less than or equal to 2048 characters");
        }

        // - Component has max length of 64 chars
        if value.component.len() > 64 {
            anyhow::bail!("component length must be less than or equal to 64 characters");
        }

        Ok(EncodedParameter {
            component: value.component,
            key: value.key,
            value: value.value,
        })
    }
}

impl From<EncodedParameter> for pb::EncodedParameter {
    fn from(value: EncodedParameter) -> Self {
        pb::EncodedParameter {
            component: value.component,
            key: value.key,
            value: value.value,
        }
    }
}

/// A set of changes to the app parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "pb::proposal::ParameterChange",
    into = "pb::proposal::ParameterChange"
)]
pub struct ParameterChange {
    pub changes: Vec<EncodedParameter>,
    pub preconditions: Vec<EncodedParameter>,
}

impl DomainType for ParameterChange {
    type Proto = pb::proposal::ParameterChange;
}

impl TryFrom<pb::proposal::ParameterChange> for ParameterChange {
    type Error = anyhow::Error;
    fn try_from(value: pb::proposal::ParameterChange) -> Result<Self, Self::Error> {
        Ok(ParameterChange {
            changes: value
                .changes
                .into_iter()
                .map(EncodedParameter::try_from)
                .collect::<Result<_, _>>()?,
            preconditions: value
                .preconditions
                .into_iter()
                .map(EncodedParameter::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<ParameterChange> for pb::proposal::ParameterChange {
    fn from(value: ParameterChange) -> Self {
        pb::proposal::ParameterChange {
            changes: value
                .changes
                .into_iter()
                .map(pb::EncodedParameter::from)
                .collect(),
            preconditions: value
                .preconditions
                .into_iter()
                .map(pb::EncodedParameter::from)
                .collect(),
            ..Default::default()
        }
    }
}

impl ParameterChange {
    /// Generates a set of encoded parameters for the given object.
    ///
    /// This is useful for generating template changes.
    pub fn encode_parameters(parameters: serde_json::Value) -> Self {
        let mut encoded_parameters = Vec::new();
        for (component, value) in parameters.as_object().into_iter().flatten() {
            for (key, value) in value.as_object().into_iter().flatten() {
                encoded_parameters.push(EncodedParameter {
                    component: component.to_string(),
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        }
        Self {
            changes: encoded_parameters.clone(),
            preconditions: encoded_parameters,
        }
    }

    /// Applies a set of changes to the "raw" app parameters.
    ///
    /// The app parameters are input as a [`serde_json::Value`] object, so that the
    /// parameter change code does not need to know about the structure of the entire
    /// application.
    ///
    /// If the changes can be successfully applied, the new app parameters are returned.
    /// By taking ownership of the input `app_parameters`, we ensure that the caller cannot
    /// access any partially-mutated app parameters in the event of an error applying one of them.
    pub fn apply_changes_raw(
        &self,
        mut app_parameters: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        for precondition in &self.preconditions {
            let expected_value = serde_json::Value::from_str(&precondition.value)
                .context("could not decode existing value as JSON value")?;

            match get_component(&mut app_parameters, precondition)?.get(&precondition.key) {
                Some(current_value) => {
                    anyhow::ensure!(
                        current_value == &expected_value,
                        "precondition failed: key {} in component {} has value {} but expected {}",
                        precondition.key,
                        precondition.component,
                        current_value,
                        expected_value
                    )
                }
                None => {
                    anyhow::bail!(
                        "precondition failed: key {} not found in component {}",
                        precondition.key,
                        precondition.component
                    );
                }
            }
        }
        for change in &self.changes {
            let component = get_component(&mut app_parameters, change)?;

            let new_value = serde_json::Value::from_str(&change.value)
                .context("could not decode new value as JSON value")?;

            // We want to insert into the map to handle the case where the existing value
            // is missing (e.g., it had a default value and so was not encoded)
            component.insert(change.key.clone(), new_value);
        }
        Ok(app_parameters)
    }
}

fn get_component<'a>(
    app_parameters: &'a mut serde_json::Value,
    change: &EncodedParameter,
) -> Result<&'a mut serde_json::Map<String, serde_json::Value>, anyhow::Error> {
    app_parameters
        .get_mut(&change.component)
        .ok_or_else(|| {
            anyhow::anyhow!("component {} not found in app parameters", change.component)
        })?
        .as_object_mut()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "expected component {} to be an object in app parameters",
                change.component
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_sdk_num::Amount;

    use crate::params::GovernanceParameters;

    const SAMPLE_JSON_PARAMETERS: &'static str = r#"
    {
        "chainId": "penumbra-testnet-deimos-6-b295771a",
        "sctParams": {
          "epochDuration": "719"
        },
        "communityPoolParams": {
          "communityPoolSpendProposalsEnabled": true
        },
        "governanceParams": {
          "proposalVotingBlocks": "17280",
          "proposalDepositAmount": {
            "lo": "10000000"
          },
          "proposalValidQuorum": "40/100",
          "proposalPassThreshold": "50/100",
          "proposalSlashThreshold": "80/100"
        },
        "ibcParams": {
          "ibcEnabled": true,
          "inboundIcs20TransfersEnabled": true,
          "outboundIcs20TransfersEnabled": true
        },
        "stakeParams": {
          "activeValidatorLimit": "80",
          "baseRewardRate": "30000",
          "slashingPenaltyMisbehavior": "10000000",
          "slashingPenaltyDowntime": "10000",
          "signedBlocksWindowLen": "10000",
          "missedBlocksMaximum": "9500",
          "minValidatorStake": {
            "lo": "1000000"
          },
          "unbondingDelay": "2158"
        },
        "feeParams": {
          "fixedGasPrices": {}
        },
        "distributionsParams": {
          "stakingIssuancePerBlock": "1"
        },
        "fundingParams": {},
        "shieldedPoolParams": {
          "fixedFmdParams": {
            "asOfBlockHeight": "1"
          }
        },
        "dexParams": {
          "isEnabled": true,
          "fixedCandidates": [
            {
              "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
            },
            {
              "inner": "reum7wQmk/owgvGMWMZn/6RFPV24zIKq3W6In/WwZgg="
            },
            {
              "inner": "HW2Eq3UZVSBttoUwUi/MUtE7rr2UU7/UH500byp7OAc="
            },
            {
              "inner": "nwPDkQq3OvLnBwGTD+nmv1Ifb2GEmFCgNHrU++9BsRE="
            },
            {
              "inner": "ypUT1AOtjfwMOKMATACoD9RSvi8jY/YnYGi46CZ/6Q8="
            },
            {
              "inner": "pmpygqUf4DL+z849rGPpudpdK/+FAv8qQ01U2C73kAw="
            },
            {
              "inner": "o2gZdbhCH70Ry+7iBhkSeHC/PB1LZhgkn7LHC2kEhQc="
            }
          ],
          "maxHops": 4,
          "maxPositionsPerPair": 10
        },
        "auctionParams": {}
      }
    "#;

    #[test]
    fn dump_encoded_parameters() {
        let parameters = serde_json::from_str(SAMPLE_JSON_PARAMETERS).unwrap();
        dbg!(&parameters);
        let encoded_parameters = ParameterChange::encode_parameters(parameters);
        for encoded_parameter in encoded_parameters.changes.iter() {
            println!("{}", serde_json::to_string(&encoded_parameter).unwrap());
        }
    }

    #[test]
    fn apply_changes_to_gov_params() {
        let old_parameters_raw: serde_json::Value =
            serde_json::from_str(SAMPLE_JSON_PARAMETERS).unwrap();

        // Make changes to the gov parameters specifically since they're
        // local to this crate so we can also inspect the decoded parameters.
        let changes = ParameterChange {
            changes: vec![
                super::EncodedParameter {
                    component: "governanceParams".to_string(),
                    key: "proposalVotingBlocks".to_string(),
                    value: r#""17281""#.to_string(),
                },
                super::EncodedParameter {
                    component: "governanceParams".to_string(),
                    key: "proposalDepositAmount".to_string(),
                    value: r#"{"lo":"10000001"}"#.to_string(),
                },
            ],
            preconditions: vec![],
        };
        let new_parameters_raw = changes
            .apply_changes_raw(old_parameters_raw.clone())
            .unwrap();

        println!(
            "{}",
            serde_json::to_string_pretty(&old_parameters_raw).unwrap()
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&new_parameters_raw).unwrap()
        );

        let old_gov_parameters_raw = old_parameters_raw["governanceParams"].clone();
        let new_gov_parameters_raw = new_parameters_raw["governanceParams"].clone();

        let old_gov_parameters: GovernanceParameters =
            serde_json::value::from_value(old_gov_parameters_raw).unwrap();
        let new_gov_parameters: GovernanceParameters =
            serde_json::value::from_value(new_gov_parameters_raw).unwrap();

        dbg!(&old_gov_parameters);
        dbg!(&new_gov_parameters);

        assert_eq!(old_gov_parameters.proposal_voting_blocks, 17280);
        assert_eq!(
            old_gov_parameters.proposal_deposit_amount,
            Amount::from(10_000_000u64)
        );
        assert_eq!(new_gov_parameters.proposal_voting_blocks, 17281);
        assert_eq!(
            new_gov_parameters.proposal_deposit_amount,
            Amount::from(10_000_001u64)
        );
    }

    #[test]
    fn protojson_rules_block_snake_case_parameter_changes() {
        let old_parameters_raw: serde_json::Value =
            serde_json::from_str(SAMPLE_JSON_PARAMETERS).unwrap();

        let bad_change_1 = ParameterChange {
            changes: vec![super::EncodedParameter {
                component: "governanceParams".to_string(),
                key: "proposal_voting_blocks".to_string(),
                value: r#""17281""#.to_string(),
            }],
            preconditions: vec![],
        };

        let new_parameters_raw = bad_change_1
            .apply_changes_raw(old_parameters_raw.clone())
            // Now we have a json Value with two keys, proposalVotingBlocks and proposal_voting_blocks
            .expect("the bad changes are still a valid json modification");

        let new_gov_parameters_raw = new_parameters_raw["governanceParams"].clone();

        // We ensure that such a json Value cannot be deserialized because the pbjson
        // Deserialize impl will treat it as a duplicate key.
        let new_gov_parameters: Result<GovernanceParameters, _> =
            serde_json::value::from_value(new_gov_parameters_raw);

        dbg!(&new_gov_parameters);

        assert!(new_gov_parameters.is_err());
    }

    #[test]
    fn preconditions_prevent_applying_changes() {
        let old_parameters_raw: serde_json::Value =
            serde_json::from_str(SAMPLE_JSON_PARAMETERS).unwrap();

        let satisfied_precondition = ParameterChange {
            preconditions: vec![super::EncodedParameter {
                component: "governanceParams".to_string(),
                key: "proposalVotingBlocks".to_string(),
                value: r#""17280""#.to_string(),
            }],
            changes: vec![super::EncodedParameter {
                component: "governanceParams".to_string(),
                key: "proposalVotingBlocks".to_string(),
                value: r#""17281""#.to_string(),
            }],
        };

        let unsatisfied_precondition = ParameterChange {
            preconditions: vec![super::EncodedParameter {
                component: "governanceParams".to_string(),
                key: "proposalVotingBlocks".to_string(),
                value: r#""17281""#.to_string(),
            }],
            changes: vec![super::EncodedParameter {
                component: "governanceParams".to_string(),
                key: "proposalVotingBlocks".to_string(),
                value: r#""17282""#.to_string(),
            }],
        };

        let satisfied_result = satisfied_precondition.apply_changes_raw(old_parameters_raw.clone());
        let unsatisfied_result =
            unsatisfied_precondition.apply_changes_raw(old_parameters_raw.clone());

        assert!(satisfied_result.is_ok());
        assert!(unsatisfied_result.is_err());
    }
}
