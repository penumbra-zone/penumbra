use core::fmt;
use std::{collections::BTreeMap, str::FromStr};

use anyhow::{Context as _, Result};
use penumbra_chain::params::ChainParameters;
use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "pb::MutableChainParameter",
    into = "pb::MutableChainParameter"
)]
pub enum MutableParam {
    UnbondingEpochs,
    ActiveValidatorLimit,
    BaseRewardRate,
    SlashingPenaltyMisbehaviorBps,
    SlashingPenaltyDowntimeBps,
    SignedBlocksWindowLen,
    MissedBlocksMaximum,
}

impl Protobuf<pb::MutableChainParameter> for MutableParam {}

impl TryFrom<pb::MutableChainParameter> for MutableParam {
    type Error = anyhow::Error;

    fn try_from(msg: pb::MutableChainParameter) -> Result<Self, Self::Error> {
        MutableParam::from_str(&msg.identifier)
    }
}

impl From<MutableParam> for pb::MutableChainParameter {
    fn from(param: MutableParam) -> Self {
        pb::MutableChainParameter {
            identifier: param.to_string(),
            description: param.description().to_string(),
        }
    }
}

impl MutableParam {
    // TODO: would be nicer as a macro but after a bit of fiddling i couldn't get it right
    pub const fn iter() -> [MutableParam; 7] {
        [
            MutableParam::UnbondingEpochs,
            MutableParam::ActiveValidatorLimit,
            MutableParam::BaseRewardRate,
            MutableParam::SlashingPenaltyMisbehaviorBps,
            MutableParam::SlashingPenaltyDowntimeBps,
            MutableParam::SignedBlocksWindowLen,
            MutableParam::MissedBlocksMaximum,
        ]
    }

    pub const fn description(&self) -> &'static str {
        match self {
            MutableParam::UnbondingEpochs => {
                "The number of epochs stake is locked up after being undelegated. Must be at least 1."
            }
            MutableParam::ActiveValidatorLimit => "The number of validators that may be in the active validator set. Must be at least 1.",
            MutableParam::BaseRewardRate => "The base reward rate for delegator pools, expressed in basis points of basis points, and accrued each epoch. Must be at least 1.",
            MutableParam::SlashingPenaltyMisbehaviorBps => "Slashing penalty specified in basis points applied to validator reward rates for as punishment for misbehavior. Must be at least 1.",
            MutableParam::SlashingPenaltyDowntimeBps => "Slashing penalty specified in basis points applied to validator reward rates as punishment for downtime. Must be at least 1.",
            MutableParam::SignedBlocksWindowLen => "Number of blocks to use as the window for detecting validator downtime. Must be at least 2 and greater than or equal to missed_blocks_maximum.",
            MutableParam::MissedBlocksMaximum => "The maximum number of blocks a validator may miss in the signed_blocks_window_len before being slashed for downtime. Must be at least 1 and less than or equal to signed_blocks_window_len.",
        }
    }
}

impl std::str::FromStr for MutableParam {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<MutableParam, Self::Err> {
        match s {
            "unbonding_epochs" => Result::Ok(MutableParam::UnbondingEpochs),
            "active_validator_limit" => Result::Ok(MutableParam::ActiveValidatorLimit),
            "base_reward_rate" => ::core::result::Result::Ok(MutableParam::BaseRewardRate),
            "slashing_penalty_misbehavior_bps" => {
                Result::Ok(MutableParam::SlashingPenaltyMisbehaviorBps)
            }
            "slashing_penalty_downtime_bps" => Result::Ok(MutableParam::SlashingPenaltyDowntimeBps),
            "signed_blocks_window_len" => Result::Ok(MutableParam::SignedBlocksWindowLen),
            "missed_blocks_maximum" => Result::Ok(MutableParam::MissedBlocksMaximum),
            _ => Err(anyhow::anyhow!("mutable parameter not found")),
        }
    }
}

impl fmt::Display for MutableParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutableParam::UnbondingEpochs => write!(f, "unbonding_epochs"),
            MutableParam::ActiveValidatorLimit => write!(f, "active_validator_limit"),
            MutableParam::BaseRewardRate => write!(f, "base_reward_rate"),
            MutableParam::SlashingPenaltyMisbehaviorBps => {
                write!(f, "slashing_penalty_misbehavior_bps")
            }
            MutableParam::SlashingPenaltyDowntimeBps => write!(f, "slashing_penalty_downtime_bps"),
            MutableParam::SignedBlocksWindowLen => write!(f, "signed_blocks_window_len"),
            MutableParam::MissedBlocksMaximum => write!(f, "missed_blocks_maximum"),
        }
    }
}

/// Validates that the proposed chain parameters are statelessly valid.
/// This performs checks that do not require access to the state.
///
/// This means they:
///
/// 1. Are mutable chain parameters
/// 2. Are within valid bounds
pub fn is_valid_stateless(new_parameters: &BTreeMap<String, String>) -> bool {
    // Validate each parameter individually.
    for (key, value) in new_parameters.iter() {
        // Check that the parameter is mutable.
        let mutable_param = match MutableParam::from_str(key) {
            Ok(param) => param,
            Err(_) => return false,
        };

        // Check that the parameter is within valid bounds.
        if !param_value_is_valid(&mutable_param, value) {
            return false;
        }
    }

    true
}

/// Validates that the proposed chain parameters are statefully valid.
///
/// This means they:
///
/// 1. Are consistent with each other
/// 2. Represent a valid change from the existing parameters
///
/// NOTE: This does not also perform the stateless validations!
pub fn is_valid_stateful(
    new_parameters: &BTreeMap<String, String>,
    old_parameters: &ChainParameters,
) -> bool {
    // Resolve the parameters into a `ChainParameters` struct.
    let new_chain_params = match resolve_parameters(new_parameters, old_parameters) {
        Ok(params) => params,
        Err(_) => return false,
    };

    // Check that the parameters are consistent with each other.
    if !params_are_consistent(&new_chain_params) {
        return false;
    }

    // Check that the parameters represent a valid change from the existing parameters.
    if !param_changes_valid(&new_chain_params, old_parameters) {
        return false;
    }

    true
}

/// Determines if the newly proposed parameter set represents a valid change from the previous parameters.
fn param_changes_valid(_new_params: &ChainParameters, _old_params: &ChainParameters) -> bool {
    // There are currently no checks to perform here, but it's feasible that we'll want to add some
    // in the future.
    true
}

/// Determines if the parameter values are consistent with one another.
fn params_are_consistent(params: &ChainParameters) -> bool {
    // Check that the signed blocks window length is greater than or equal to the missed blocks maximum.
    if params.signed_blocks_window_len < params.missed_blocks_maximum {
        return false;
    }

    true
}

/// Collates old parameters with new parameters to produce a new `ChainParameters` struct.
pub fn resolve_parameters(
    new_parameters: &BTreeMap<String, String>,
    old_parameters: &ChainParameters,
) -> Result<ChainParameters> {
    let mut new_chain_params = old_parameters.clone();

    for (key, value) in new_parameters.iter() {
        let mutable_param = MutableParam::from_str(key).context("invalid parameter")?;

        match mutable_param {
            MutableParam::UnbondingEpochs => {
                new_chain_params.unbonding_epochs = value.parse().context("invalid value")?
            }
            MutableParam::ActiveValidatorLimit => {
                new_chain_params.active_validator_limit = value.parse().context("invalid value")?
            }
            MutableParam::BaseRewardRate => {
                new_chain_params.base_reward_rate = value.parse().context("invalid value")?
            }
            MutableParam::SlashingPenaltyMisbehaviorBps => {
                new_chain_params.slashing_penalty_misbehavior_bps =
                    value.parse().context("invalid value")?
            }
            MutableParam::SlashingPenaltyDowntimeBps => {
                new_chain_params.slashing_penalty_downtime_bps =
                    value.parse().context("invalid value")?
            }
            MutableParam::SignedBlocksWindowLen => {
                new_chain_params.signed_blocks_window_len =
                    value.parse().context("invalid value")?
            }
            MutableParam::MissedBlocksMaximum => {
                new_chain_params.missed_blocks_maximum = value.parse().context("invalid value")?
            }
        }
    }

    Ok(new_chain_params)
}

/// Validates that the value for the given parameter is within valid bounds.
fn param_value_is_valid(param: &MutableParam, value: &str) -> bool {
    // TODO: do these constraints make sense? They're set to the absolute bounds
    // right now, but maybe there are more sensible ranges that we should enforce?
    match param {
        MutableParam::UnbondingEpochs => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Unbonding epochs must be at least 1.
            value >= 1
        }
        MutableParam::ActiveValidatorLimit => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Active validator limit must be at least 2.
            value >= 2
        }
        MutableParam::BaseRewardRate => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Base reward rate must be at least 1.
            value >= 1
        }
        MutableParam::SlashingPenaltyMisbehaviorBps => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Slashing penalty for misbehavior must be at least 1.
            value >= 1
        }
        MutableParam::SlashingPenaltyDowntimeBps => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Slashing penalty for downtime must be at least 1.
            value >= 1
        }
        MutableParam::SignedBlocksWindowLen => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Signed blocks window length must be at least 2.
            value >= 2
        }
        MutableParam::MissedBlocksMaximum => {
            let value = match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            // Missed blocks maximum must be at least 1.
            value >= 1
        }
    }
}
