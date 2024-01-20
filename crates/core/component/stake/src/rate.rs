//! Staking reward and delegation token exchange rates.

use penumbra_num::fixpoint::U128x128;
use penumbra_num::Amount;
use penumbra_proto::core::component::stake::v1alpha1::CurrentValidatorRateResponse;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{validator::State, FundingStream, IdentityKey};
use crate::{Delegate, Penalty, Undelegate};

/// Describes a validator's reward rate and voting power in some epoch.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::RateData", into = "pb::RateData")]
pub struct RateData {
    /// The validator's identity key.
    pub identity_key: IdentityKey,
    /// The index of the epoch for which this rate is valid.
    pub epoch_index: u64,
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
    /// The stateless checks in the [`ValidatorDefinition`] action handler
    /// should prevent this from happening.
    pub fn next(
        &self,
        next_base_rate: &BaseRateData,
        funding_streams: &[FundingStream],
        validator_state: &State,
    ) -> RateData {
        let previous_rate = self;

        if let State::Active = validator_state {
            // Compute the validator's total commission rate in basis points.
            let commission_rate_bps = funding_streams
                .iter()
                .fold(0u64, |total, stream| total + stream.rate_bps() as u64);

            if commission_rate_bps > 1_0000 {
                // We should never hit this branch: validator funding streams should be verified not to
                // sum past 100% in the state machine's validation of registration of new funding
                // streams
                panic!("commission rate sums to > 100%")
            }

            /* ************ Compute the validator reward rate **************** */
            let commission_rate_bps = U128x128::from(commission_rate_bps);
            let bps_scaling = U128x128::from(1_0000u128);
            let scaling_factor = U128x128::from(1_0000_0000u128);
            let next_base_reward_rate = U128x128::from(next_base_rate.base_reward_rate);

            // TODO(erwan): this PR focus on using `Amount`s and `fixnum`.
            // But, this has to be cleaned up, it's a mess and frustrating to work with.
            let scaled_commision_factor =
                (commission_rate_bps * bps_scaling).expect("does not overflow");
            let scaled_commission_rate = (scaling_factor - scaled_commision_factor)
                .expect("scaled_commission_factor > scaling_factor");

            let unscaled_validator_reward_rate =
                (scaled_commission_rate * next_base_reward_rate).expect("does not overflow");

            let next_validator_reward_rate: Amount = (unscaled_validator_reward_rate
                / scaling_factor)
                .expect("scaling factor is nonzero")
                .round_down()
                .try_into()
                .expect("rounding down gives an integral type");
            /* ***************************************************************** */

            /* ************ Compute the validator exchange rate **************** */
            // The conversion rate between delegation tokens and unbonded stake.
            let next_validator_reward_rate = U128x128::from(next_validator_reward_rate);
            let previous_validator_exchange_rate =
                U128x128::from(previous_rate.validator_exchange_rate);
            let scaling_factor = U128x128::from(1_0000_0000u128);

            let effective_reward_rate = (next_validator_reward_rate
                * previous_validator_exchange_rate)
                .expect("does not overflow");
            let validator_exchange_rate = (effective_reward_rate / scaling_factor)
                .expect("scaling factor is nonzero")
                .round_down()
                .try_into()
                .expect("rounding down gives an integral type");
            /* ***************************************************************** */

            let next_validator_reward_rate =
                next_validator_reward_rate.try_into().expect("integral");

            RateData {
                identity_key: previous_rate.identity_key.clone(),
                epoch_index: previous_rate.epoch_index + 1,
                validator_reward_rate: next_validator_reward_rate,
                validator_exchange_rate,
            }
        } else {
            // Non-Active validator states result in a constant rate. This means
            // the next epoch's rate is set to the current rate.
            RateData {
                identity_key: previous_rate.identity_key.clone(),
                epoch_index: previous_rate.epoch_index + 1,
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
        // validator_exchange_rate fits in 32 bits, but unbonded_amount is 64-bit;
        // upconvert to u128 intermediates and panic if the result is too large (unlikely)
        let scaling_factor = U128x128::from(1_0000_0000u128);
        let unbonded_amount = U128x128::from(unbonded_amount);
        let validator_exchange_rate = U128x128::from(self.validator_exchange_rate);
        let unscaled_delegation_amount = (unbonded_amount / validator_exchange_rate)
            .expect("validator exchange rate is nonzero");
        let delegation_amount =
            (unscaled_delegation_amount * scaling_factor).expect("does not overflow");
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
        let delegation_amount = U128x128::from(delegation_amount);
        let validator_exchange_rate = U128x128::from(self.validator_exchange_rate);
        let scaling_factor = U128x128::from(1_0000_0000u128);
        let unscaled_unbonded_amount =
            (delegation_amount * validator_exchange_rate).expect("does not overflow");
        let unbonded_amount =
            (unscaled_unbonded_amount / scaling_factor).expect("does not overflow");
        unbonded_amount
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type")
    }

    /// Computes the validator's voting power at this epoch given the total supply of the
    /// validator's delegation tokens.
    pub fn voting_power(
        &self,
        _total_delegation_tokens: u128,
        _base_rate_data: &BaseRateData,
    ) -> u64 {
        // ((total_delegation_tokens * self.validator_exchange_rate as u128)
        //     / base_rate_data.base_exchange_rate as u128)
        //     .try_into()
        //     .expect("voting power should fit in u64")
        todo!("MERGEBLOCK(erwan): circle back to do this cleanly")
    }

    /// Uses this `RateData` to build a `Delegate` transaction action that
    /// delegates `unbonded_amount` of the staking token.
    pub fn build_delegate(&self, unbonded_amount: Amount) -> Delegate {
        Delegate {
            delegation_amount: self.delegation_amount(unbonded_amount).into(),
            epoch_index: self.epoch_index,
            unbonded_amount: unbonded_amount.into(),
            validator_identity: self.identity_key.clone(),
        }
    }

    /// Uses this `RateData` to build an `Undelegate` transaction action that
    /// undelegates `delegation_amount` of the validator's delegation tokens.
    pub fn build_undelegate(&self, delegation_amount: Amount) -> Undelegate {
        Undelegate {
            start_epoch_index: self.epoch_index,
            delegation_amount,
            unbonded_amount: self.unbonded_amount(delegation_amount.into()).into(),
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
    /// Compute the base rate data for the epoch following the current one,
    /// given the next epoch's base reward rate.
    pub fn next(&self, next_base_reward_rate: Amount) -> BaseRateData {
        let base_reward_rate_fp = U128x128::from(self.base_reward_rate);
        let next_base_reward_rate_fp = U128x128::from(next_base_reward_rate);
        let scaling_factor = U128x128::from(1_0000_0000u128);

        let unscaled_combined_rate =
            (base_reward_rate_fp * next_base_reward_rate_fp).expect("does not overflow");
        let combined_rate =
            (unscaled_combined_rate / scaling_factor).expect("scaling factor is nonzero");

        let next_base_exchange_rate = (combined_rate + next_base_reward_rate_fp)
            .expect("does not overflow")
            .round_down()
            .try_into()
            .expect("rounding down gives an integral type");

        BaseRateData {
            base_exchange_rate: next_base_exchange_rate,
            base_reward_rate: next_base_reward_rate,
            epoch_index: self.epoch_index + 1,
        }
    }
}

impl DomainType for RateData {
    type Proto = pb::RateData;
}

impl From<RateData> for pb::RateData {
    fn from(_v: RateData) -> Self {
        // pb::RateData {
        //     identity_key: Some(v.identity_key.into()),
        //     epoch_index: v.epoch_index,
        //     validator_reward_rate: v.validator_reward_rate,
        //     validator_exchange_rate: v.validator_exchange_rate,
        // }
        todo!("MERGEBLOCK(erwan): change the proto definitions to use amounts")
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
            epoch_index: v.epoch_index,
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
        let sk = rdsa::SigningKey::new(OsRng);
        let ik = IdentityKey((&sk).into());

        let rate_data = RateData {
            identity_key: ik,
            epoch_index: 0,
            validator_reward_rate: 1_0000_0000u128.into(),
            validator_exchange_rate: 2_0000_0000u128.into(),
        };
        // 10%
        let penalty = Penalty::from_percent(10);
        let slashed = rate_data.slash(penalty);
        assert_eq!(slashed.validator_exchange_rate, 1_8000_0000u128.into());
    }
}
