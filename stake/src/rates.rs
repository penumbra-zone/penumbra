const IMPLICIT_DENOMINATOR: u64 = 10u64.pow(8);

pub struct BaseExchangeRate {
    exchange_rate: u64,
}

impl BaseExchangeRate {
    fn next_exchange_rate(self, epoch_base_rate: u64) -> BaseExchangeRate {
        let exchange_rate =
            (self.exchange_rate * (IMPLICIT_DENOMINATOR + epoch_base_rate)) / IMPLICIT_DENOMINATOR;

        BaseExchangeRate { exchange_rate }
    }
}

pub struct ValidatorRewardRate {
    pub reward_rate: u64,
}

impl ValidatorRewardRate {
    fn next_reward_rate(
        self,
        epoch_commission_rate: u16,
        epoch_base_rate: u64,
    ) -> ValidatorRewardRate {
        let reward_rate = ((IMPLICIT_DENOMINATOR - (epoch_commission_rate as u64 * 10u64.pow(4)))
            * epoch_base_rate)
            / IMPLICIT_DENOMINATOR;

        ValidatorRewardRate { reward_rate }
    }
}

pub struct ValidatorExchangeRate {
    exchange_rate: u64,
}

impl ValidatorExchangeRate {
    fn next_exchange_rate(
        self,
        epoch_validator_reward_rate: ValidatorRewardRate,
    ) -> ValidatorExchangeRate {
        let exchange_rate = (self.exchange_rate
            * (IMPLICIT_DENOMINATOR + epoch_validator_reward_rate.reward_rate))
            / IMPLICIT_DENOMINATOR;

        ValidatorExchangeRate { exchange_rate }
    }
}

/// compute the voting power adjustment function
pub fn voting_power_adjustment(
    base_exchange_rate: BaseExchangeRate,
    val_exchange_rate: ValidatorExchangeRate,
) -> u64 {
    (val_exchange_rate.exchange_rate * IMPLICIT_DENOMINATOR) / base_exchange_rate.exchange_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_exchange_rate() {
        let base_exg = BaseExchangeRate {
            exchange_rate: 1000,
        };

        println!("{}", base_exg.next_exchange_rate(100).exchange_rate);
    }
}
