//! Staking reward and delegation token exchange rates.

use penumbra_sdk_num::fixpoint::U128x128;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::stake::v1::CurrentValidatorRateResponse;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use penumbra_sdk_sct::epoch::Epoch;
use serde::{Deserialize, Serialize};

use crate::{validator::State, FundingStream, IdentityKey};
use crate::{Delegate, Penalty, Undelegate, BPS_SQUARED_SCALING_FACTOR};

/// Describes a validator's reward rate and voting power in some epoch.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::RateData", into = "pb::RateData")]
pub struct RateData {
    /// The validator's identity key.
    pub identity_key: IdentityKey,
    /// The validator-specific reward rate.
    pub validator_reward_rate: Amount,
    /// The validator-specific exchange rate.
    pub validator_exchange_rate: Amount,
}

impl RateData {
    /// Compute the validator rate data for the next epoch.
    ///
    /// # Panics
    /// This method panics if the validator's funding streams exceed 100%.
    /// The stateless checks in the [`Definition`](crate::validator::Definition) action handler
    /// should prevent this from happening.
    pub fn next_epoch(
        &self,
        next_base_rate: &BaseRateData,
        funding_streams: &[FundingStream],
        validator_state: &State,
    ) -> RateData {
        let previous_rate = self;

        if let State::Active = validator_state {
            // Compute the validator's total commission rate in basis points.
            let validator_commission_bps = funding_streams
                .iter()
                .fold(0u64, |total, stream| total + stream.rate_bps() as u64);

            if validator_commission_bps > 1_0000 {
                // We should never hit this branch: validator funding streams should be verified not to
                // sum past 100% in the state machine's validation of registration of new funding
                // streams
                panic!("commission rate sums to > 100%")
            }

            // Rate data is represented with an implicit scaling factor of 1_0000_0000.
            // To make the calculations more readable, we use `U128x128` to represent
            // the intermediate descaled values. As a last step, we scaled them back
            // using [`BPS_SQUARED_SCALING_FACTOR`] and round down to an [`Amount`].

            /* Setting up constants and unrolling scaling factors */
            let one = U128x128::from(1u128);
            let max_bps = U128x128::from(1_0000u128);

            let validator_commission_bps = U128x128::from(validator_commission_bps);
            let next_base_reward_rate = U128x128::from(next_base_rate.base_reward_rate);
            let previous_validator_exchange_rate =
                U128x128::from(previous_rate.validator_exchange_rate);

            let validator_commission =
                (validator_commission_bps / max_bps).expect("max_bps is nonzero");
            let next_base_reward_rate = (next_base_reward_rate / *BPS_SQUARED_SCALING_FACTOR)
                .expect("scaling factor is nonzero");
            let previous_validator_exchange_rate = (previous_validator_exchange_rate
                / *BPS_SQUARED_SCALING_FACTOR)
                .expect("scaling factor is nonzero");
            /* ************************************************* */

            /* ************ Compute the validator reward rate **************** */
            tracing::debug!(%validator_commission, %next_base_reward_rate, "computing validator reward rate");
            let commission_factor =
                (one - validator_commission).expect("0 <= validator_commission_bps <= 1");
            tracing::debug!(%commission_factor, "complement commission rate");

            let next_validator_reward_rate =
                (next_base_reward_rate * commission_factor).expect("does not overflow");
            tracing::debug!(%next_validator_reward_rate, "validator reward rate");
            /* ***************************************************************** */

            /* ************ Compute the validator exchange rate **************** */
            tracing::debug!(%next_validator_reward_rate, %previous_validator_exchange_rate, "computing validator exchange rate");

            let reward_growth_factor =
                (one + next_validator_reward_rate).expect("does not overflow");
            let next_validator_exchange_rate = (previous_validator_exchange_rate
                * reward_growth_factor)
                .expect("does not overflow");
            tracing::debug!(%next_validator_exchange_rate, "computed the validator exchange rate");
            /* ***************************************************************** */

            /* Rescale the rate data using the fixed point scaling factor */
            let next_validator_reward_rate = (next_validator_reward_rate
                * *BPS_SQUARED_SCALING_FACTOR)
                .expect("rate is between 0 and 1")
                .round_down()
                .try_into()
                .expect("rounding down gives an integral type");
            let next_validator_exchange_rate = (next_validator_exchange_rate
                * *BPS_SQUARED_SCALING_FACTOR)
                .expect("rate is between 0 and 1")
                .round_down()
                .try_into()
                .expect("rounding down gives an integral type");
            /* ************************************************************* */

            RateData {
                identity_key: previous_rate.identity_key.clone(),
                validator_reward_rate: next_validator_reward_rate,
                validator_exchange_rate: next_validator_exchange_rate,
            }
        } else {
            // Non-Active validator states result in a constant rate. This means
            // the next epoch's rate is set to the current rate.
            RateData {
                identity_key: previous_rate.identity_key.clone(),
                validator_reward_rate: previous_rate.validator_reward_rate,
                validator_exchange_rate: previous_rate.validator_exchange_rate,
            }
        }
    }

    /// Computes the amount of delegation tokens corresponding to the given amount of unbonded stake.
    ///
    /// # Warning
    ///
    /// Given a pair `(delegation_amount, unbonded_amount)` and `rate_data`, it's possible to have
    /// ```rust,ignore
    /// delegation_amount == rate_data.delegation_amount(unbonded_amount)
    /// ```
    /// or
    /// ```rust,ignore
    /// unbonded_amount == rate_data.unbonded_amount(delegation_amount)
    /// ```
    /// but in general *not both*, because the computation involves rounding.
    pub fn delegation_amount(&self, unbonded_amount: Amount) -> Amount {
        // Setup:
        let unbonded_amount = U128x128::from(unbonded_amount);
        let validator_exchange_rate = U128x128::from(self.validator_exchange_rate);

        // Remove scaling factors:
        let validator_exchange_rate = (validator_exchange_rate / *BPS_SQUARED_SCALING_FACTOR)
            .expect("scaling factor is nonzero");
        if validator_exchange_rate == U128x128::from(0u128) {
            // If the exchange rate is zero, the delegation amount is also zero.
            // This is extremely unlikely to be hit in practice, but it's a valid
            // edge case that a test might want to cover.
            return 0u128.into();
        }

        /* **************** Compute the corresponding delegation size *********************** */

        let delegation_amount = (unbonded_amount / validator_exchange_rate)
            .expect("validator exchange rate is nonzero");
        /* ********************************************************************************** */

        delegation_amount
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type")
    }

    pub fn slash(&self, penalty: Penalty) -> Self {
        let mut slashed = self.clone();
        // This will automatically produce a ratio which is multiplied by 1_0000_0000, and so
        // rounding down does what we want.
        let penalized_exchange_rate: Amount = penalty
            .apply_to(self.validator_exchange_rate)
            .round_down()
            .try_into()
            .expect("multiplying will not overflow");
        slashed.validator_exchange_rate = penalized_exchange_rate;
        slashed
    }

    /// Computes the amount of unbonded stake corresponding to the given amount of delegation tokens.
    ///
    /// # Warning
    ///
    /// Given a pair `(delegation_amount, unbonded_amount)` and `rate_data`, it's possible to have
    /// ```rust,ignore
    /// delegation_amount == rate_data.delegation_amount(unbonded_amount)
    /// ```
    /// or
    /// ```rust,ignore
    /// unbonded_amount == rate_data.unbonded_amount(delegation_amount)
    /// ```
    /// but in general *not both*, because the computation involves rounding.
    pub fn unbonded_amount(&self, delegation_amount: Amount) -> Amount {
        // Setup:
        let delegation_amount = U128x128::from(delegation_amount);
        let validator_exchange_rate = U128x128::from(self.validator_exchange_rate);

        // Remove scaling factors:
        let validator_exchange_rate = (validator_exchange_rate / *BPS_SQUARED_SCALING_FACTOR)
            .expect("scaling factor is nonzero");

        /* **************** Compute the unbonded amount *********************** */
        (delegation_amount * validator_exchange_rate)
            .expect("does not overflow")
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type")
    }

    /// Compute the voting power of the validator given the size of its delegation pool.
    pub fn voting_power(&self, delegation_pool_size: Amount) -> Amount {
        // Setup:
        let delegation_pool_size = U128x128::from(delegation_pool_size);
        let validator_exchange_rate = U128x128::from(self.validator_exchange_rate);

        // Remove scaling factors:
        let validator_exchange_rate = (validator_exchange_rate / *BPS_SQUARED_SCALING_FACTOR)
            .expect("scaling factor is nonzero");

        /* ************************ Convert the delegation tokens to staking tokens ******************** */
        let voting_power = (delegation_pool_size * validator_exchange_rate)
            .expect("does not overflow")
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type");
        /* ******************************************************************************************* */

        voting_power
    }

    /// Uses this `RateData` to build a `Delegate` transaction action that
    /// delegates `unbonded_amount` of the staking token.
    pub fn build_delegate(&self, epoch: Epoch, unbonded_amount: Amount) -> Delegate {
        Delegate {
            delegation_amount: self.delegation_amount(unbonded_amount),
            epoch_index: epoch.index,
            unbonded_amount,
            validator_identity: self.identity_key.clone(),
        }
    }

    /// Uses this `RateData` to build an `Undelegate` transaction action that
    /// undelegates `delegation_amount` of the validator's delegation tokens.
    pub fn build_undelegate(&self, start_epoch: Epoch, delegation_amount: Amount) -> Undelegate {
        Undelegate {
            from_epoch: start_epoch,
            delegation_amount,
            unbonded_amount: self.unbonded_amount(delegation_amount),
            validator_identity: self.identity_key.clone(),
        }
    }
}

/// Describes the base reward and exchange rates in some epoch.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::BaseRateData", into = "pb::BaseRateData")]
pub struct BaseRateData {
    /// The index of the epoch for which this rate is valid.
    pub epoch_index: u64,
    /// The base reward rate.
    pub base_reward_rate: Amount,
    /// The base exchange rate.
    pub base_exchange_rate: Amount,
}

impl BaseRateData {
    /// Compute the next epoch's base rate.
    pub fn next_epoch(&self, next_base_reward_rate: Amount) -> BaseRateData {
        // Setup:
        let prev_base_exchange_rate = U128x128::from(self.base_exchange_rate);
        let next_base_reward_rate_scaled = next_base_reward_rate.clone();
        let next_base_reward_rate = U128x128::from(next_base_reward_rate);
        let one = U128x128::from(1u128);

        // Remove scaling factors:
        let prev_base_exchange_rate = (prev_base_exchange_rate / *BPS_SQUARED_SCALING_FACTOR)
            .expect("scaling factor is nonzero");
        let next_base_reward_rate_fp = (next_base_reward_rate / *BPS_SQUARED_SCALING_FACTOR)
            .expect("scaling factor is nonzero");

        // Compute the reward growth factor:
        let reward_growth_factor = (one + next_base_reward_rate_fp).expect("does not overflow");

        /* ********* Compute the base exchange rate for the next epoch ****************** */
        let next_base_exchange_rate =
            (prev_base_exchange_rate * reward_growth_factor).expect("does not overflow");
        /* ****************************************************************************** */

        // Rescale the exchange rate:
        let next_base_exchange_rate_scaled = (next_base_exchange_rate
            * *BPS_SQUARED_SCALING_FACTOR)
            .expect("rate is between 0 and 1")
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type");

        BaseRateData {
            base_exchange_rate: next_base_exchange_rate_scaled,
            base_reward_rate: next_base_reward_rate_scaled,
            epoch_index: self.epoch_index + 1,
        }
    }
}

impl DomainType for RateData {
    type Proto = pb::RateData;
}

impl From<RateData> for pb::RateData {
    #[allow(deprecated)]
    fn from(v: RateData) -> Self {
        pb::RateData {
            identity_key: Some(v.identity_key.into()),
            epoch_index: 0,
            validator_reward_rate: Some(v.validator_reward_rate.into()),
            validator_exchange_rate: Some(v.validator_exchange_rate.into()),
        }
    }
}

impl TryFrom<pb::RateData> for RateData {
    type Error = anyhow::Error;
    fn try_from(v: pb::RateData) -> Result<Self, Self::Error> {
        Ok(RateData {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key"))?
                .try_into()?,
            validator_reward_rate: v
                .validator_reward_rate
                .ok_or_else(|| anyhow::anyhow!("empty validator reward rate in RateData message"))?
                .try_into()?,
            validator_exchange_rate: v
                .validator_exchange_rate
                .ok_or_else(|| {
                    anyhow::anyhow!("empty validator exchange rate in RateData message")
                })?
                .try_into()?,
        })
    }
}

impl DomainType for BaseRateData {
    type Proto = pb::BaseRateData;
}

impl From<BaseRateData> for pb::BaseRateData {
    fn from(rate: BaseRateData) -> Self {
        pb::BaseRateData {
            epoch_index: rate.epoch_index,
            base_reward_rate: Some(rate.base_reward_rate.into()),
            base_exchange_rate: Some(rate.base_exchange_rate.into()),
        }
    }
}

impl TryFrom<pb::BaseRateData> for BaseRateData {
    type Error = anyhow::Error;
    fn try_from(v: pb::BaseRateData) -> Result<Self, Self::Error> {
        Ok(BaseRateData {
            epoch_index: v.epoch_index,
            base_reward_rate: v
                .base_reward_rate
                .ok_or_else(|| anyhow::anyhow!("empty base reward rate in BaseRateData message"))?
                .try_into()?,
            base_exchange_rate: v
                .base_exchange_rate
                .ok_or_else(|| anyhow::anyhow!("empty base exchange rate in BaseRateData message"))?
                .try_into()?,
        })
    }
}

impl From<RateData> for CurrentValidatorRateResponse {
    fn from(r: RateData) -> Self {
        CurrentValidatorRateResponse {
            data: Some(r.into()),
        }
    }
}

impl TryFrom<CurrentValidatorRateResponse> for RateData {
    type Error = anyhow::Error;

    fn try_from(value: CurrentValidatorRateResponse) -> Result<Self, Self::Error> {
        value
            .data
            .ok_or_else(|| anyhow::anyhow!("empty CurrentValidatorRateResponse message"))?
            .try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decaf377_rdsa as rdsa;
    use rand_core::OsRng;

    #[test]
    fn slash_rate_by_penalty() {
        let vk = rdsa::VerificationKey::from(rdsa::SigningKey::new(OsRng));
        let ik = IdentityKey(vk.into());

        let rate_data = RateData {
            identity_key: ik,
            validator_reward_rate: 1_0000_0000u128.into(),
            validator_exchange_rate: 2_0000_0000u128.into(),
        };
        // 10%
        let penalty = Penalty::from_percent(10);
        let slashed = rate_data.slash(penalty);
        assert_eq!(slashed.validator_exchange_rate, 1_8000_0000u128.into());
    }
}
