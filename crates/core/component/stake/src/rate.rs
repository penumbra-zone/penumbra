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
    pub validator_reward_rate: U128x128,
    /// The validator-specific exchange rate.
    pub validator_exchange_rate: U128x128,
}

impl RateData {
    /// Compute the validator rate data for the epoch following the current one.
    pub fn next(
        &self,
        base_rate_data: &BaseRateData,
        funding_streams: &[FundingStream],
        validator_state: &State,
    ) -> RateData {
        let prev = self;

        if let State::Active = validator_state {
            // compute the validator's total commission
            let commission_rate_bps = funding_streams
                .iter()
                .fold(0u64, |total, stream| total + stream.rate_bps() as u64);

            if commission_rate_bps > 1_0000 {
                // we should never hit this branch: validator funding streams should be verified not to
                // sum past 100% in the state machine's validation of registration of new funding
                // streams
                panic!("commission rate sums to > 100%")
            }

            // Compute the validator reward rate for the next epoch, it is the product of the base
            // reward rate and the complement of the validator's commission rate, or:
            // `(1 - commission_rate) * base_reward_rate`
            let one: U128x128 = 1u128.into();
            let commission_rate = U128x128::ratio(commission_rate_bps, 10000).expect("infaillible");
            let residual_commission_rate = (one - commission_rate).expect("commission rate <= 1");
            let validator_reward_rate = (residual_commission_rate
                * base_rate_data.base_reward_rate)
                .expect("reward rate <= 1");

            // Compute the validator exchange rate for the next epoch, it is the product of the
            // validator's previous exchange rate and the validator's reward rate, or:
            // `prev.validator_exchange_rate * (1 + validator_reward_rate)`
            let growth_factor_reward_rate = (validator_reward_rate + one)
                .expect("validator_reward_rate <= 1 so this cannot overflow");
            let validator_exchange_rate = (prev.validator_exchange_rate
                * growth_factor_reward_rate)
                .expect("exchange rate is small");

            RateData {
                identity_key: prev.identity_key.clone(),
                epoch_index: prev.epoch_index + 1,
                validator_reward_rate,
                validator_exchange_rate,
            }
        } else {
            // Non-Active validator states result in a constant rate. This means
            // the next epoch's rate is set to the current rate.
            RateData {
                identity_key: prev.identity_key.clone(),
                epoch_index: prev.epoch_index + 1,
                validator_reward_rate: prev.validator_reward_rate,
                validator_exchange_rate: prev.validator_exchange_rate,
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
        let unbonded_amount_fp: U128x128 = unbonded_amount.into();
        let delegation_amount = (unbonded_amount_fp / self.validator_exchange_rate)
            .expect("exchange rate is close to 1"); // MERGEBLOCK(erwan): review the `expect`s statements

        delegation_amount
            .round_down()
            .try_into()
            .expect("integral value")
    }

    /// Return a new [`RateData`] with the exchange rate slashed by the given penalty.
    pub fn slash(&self, penalty: Penalty) -> Self {
        let one = U128x128::from(1u128);

        // MERGEBLOCK(erwan): for now, used a stubbed penalty. This is because I want to be able
        // to do side-by-side comparison of the `Penalty` implementation with the circuit change branch.
        let stub_penalty = U128x128::from(1u128);
        // replace later with: `let penalty = penalty.0;`

        let complement_penalty = (one - stub_penalty).expect("penalty <= 1");
        let slashed_validator_exchange_rate = (self.validator_exchange_rate * complement_penalty)
            .expect("exchange rate is close to 1");

        let mut slashed_rate = self.clone();
        slashed_rate.validator_exchange_rate = slashed_validator_exchange_rate;
        slashed_rate
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
        let delegation_amount_fp: U128x128 = delegation_amount.into();
        let unbonded_amount = (delegation_amount_fp * self.validator_exchange_rate)
            .expect("exchange rate is close to 1");

        unbonded_amount
            .round_down()
            .try_into()
            .expect("integral value")
    }

    /// Compute the validator's voting power at the current epoch, based on the size of
    /// its delegation pool.
    pub fn voting_power(
        &self,
        total_delegation_tokens: Amount,
        base_rate_data: &BaseRateData,
    ) -> Amount {
        let delegation_pool_size: U128x128 = total_delegation_tokens.into();
        // Compute the amount of staking tokens that the delegation pool corresponds to.
        let total_staking_tokens = (delegation_pool_size * base_rate_data.base_exchange_rate)
            .expect("exchange rate is close to 1");
        // We are not done yet, we have to normalize based on the base exchange rate.
        let voting_power = (total_staking_tokens / base_rate_data.base_exchange_rate)
            .expect("exchange rate is close to 1");

        voting_power
            .round_down()
            .try_into()
            .expect("integral value")
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
    pub base_reward_rate: U128x128,
    /// The base exchange rate.
    pub base_exchange_rate: U128x128,
}

impl BaseRateData {
    /// Compute the base rate data for the epoch following the current one,
    /// given the next epoch's base reward rate.
    pub fn next(&self, base_reward_rate: U128x128) -> BaseRateData {
        let base_exchange_rate =
            (self.base_exchange_rate * base_reward_rate).expect("rates are <= 1");
        BaseRateData {
            base_exchange_rate,
            base_reward_rate,
            epoch_index: self.epoch_index + 1,
        }
    }
}

impl DomainType for RateData {
    type Proto = pb::RateData;
}

impl From<RateData> for pb::RateData {
    fn from(v: RateData) -> Self {
        pb::RateData {
            identity_key: Some(v.identity_key.into()),
            epoch_index: v.epoch_index,
            validator_reward_rate: v.validator_reward_rate,
            validator_exchange_rate: v.validator_exchange_rate,
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
            epoch_index: v.epoch_index,
            validator_reward_rate: v.validator_reward_rate,
            validator_exchange_rate: v.validator_exchange_rate,
        })
    }
}

impl DomainType for BaseRateData {
    type Proto = pb::BaseRateData;
}

impl From<BaseRateData> for pb::BaseRateData {
    fn from(v: BaseRateData) -> Self {
        pb::BaseRateData {
            epoch_index: v.epoch_index,
            base_reward_rate: v.base_reward_rate,
            base_exchange_rate: v.base_exchange_rate,
        }
    }
}

impl TryFrom<pb::BaseRateData> for BaseRateData {
    type Error = anyhow::Error;
    fn try_from(v: pb::BaseRateData) -> Result<Self, Self::Error> {
        Ok(BaseRateData {
            epoch_index: v.epoch_index,
            base_reward_rate: v.base_reward_rate,
            base_exchange_rate: v.base_exchange_rate,
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
            validator_reward_rate: 1u128.into(),
            validator_exchange_rate: 2u128.into(),
        };
        // 10%
        let penalty = Penalty::from_percent(10);
        let slashed = rate_data.slash(penalty);
        assert_eq!(
            slashed.validator_exchange_rate,
            U128x128::ratio(18u128, 100)
        );
    }
}
