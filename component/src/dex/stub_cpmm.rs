use penumbra_crypto::dex::lp::Reserves;

/// A stub constant-product market maker, used to exercise the swap
/// functionality before we implement the real DEX backend.
pub struct StubCpmm {
    pub reserves: Reserves,
}

impl StubCpmm {
    /// Trade $\Delta_1$ of asset 1 for $\Lambda_2$ of asset 2.
    pub fn trade_1_to_2(&mut self, delta_1: u64) -> u64 {
        // (R_1 + \Delta_1) (R_2 - \Lambda_2) = R_1 R_2
        // R_1 R_2 + \Delta_1 R_2 - \Lambda_2 (R_1 + \Delta_1) = R_1 R_2
        // \Delta_1 R_2 = \Lambda_2 (R_1 + \Delta_1)
        // (\Delta_1 R_2 ) / (R_1 + \Delta_1) = \Lambda_2

        let Reserves { r1, r2 } = self.reserves;

        let num = r2 as u128 * delta_1 as u128;
        let den = r1 as u128 + delta_1 as u128;
        // Not that correctness really matters here,
        // but this rounds *down* the output amount.
        let lambda_2 = (num / den) as u64;

        self.reserves = Reserves {
            r1: r1 + delta_1,
            r2: r2 - lambda_2,
        };

        lambda_2
    }

    /// Trade $\Delta_2$ of asset 2 for $\Lambda_1$ of asset 1.
    pub fn trade_2_to_1(&mut self, delta_2: u64) -> u64 {
        // (R_1 - \Lambda_1) (R_2 + \Delta_2) = R_1 R_2
        // R_1 R_2 + \Delta_2 R_1 - \Lambda_1 (R_2 + \Delta_2) = R_1 R_2
        // \Delta_2 R_1 = \Lambda_1 (R_2 + \Delta_2)
        // (\Delta_2 R_1 ) / (R_2 + \Delta_2) = \Lambda_1

        let Reserves { r1, r2 } = self.reserves;

        let num = r1 as u128 * delta_2 as u128;
        let den = r2 as u128 + delta_2 as u128;
        // Not that correctness really matters here,
        // but this rounds *down* the output amount.
        let lambda_1 = (num / den) as u64;

        self.reserves = Reserves {
            r1: r1 - lambda_1,
            r2: r2 + delta_2,
        };

        lambda_1
    }

    /// Trade $(\Delta_1, \Delta_2)$ against the CPMM to get outputs $(\Lambda_1, \Lambda_2)$, netting cross flow at the current price.
    pub fn trade_netted(&mut self, delta: (u64, u64)) -> (u64, u64) {
        match delta {
            (delta_1, 0) => (0, self.trade_1_to_2(delta_1)),
            (0, delta_2) => (self.trade_2_to_1(delta_2), 0),
            (delta_1, delta_2) => {
                // We want to net out the cross flow at the current price.
                // To do that, we need to determine which input is "bigger",
                // and will absorb the other direction's flow.

                let Reserves { r1, r2 } = self.reserves;

                // The amount of asset 2 we get from asset 1 at current prices.
                let lambda_2_netted = ((delta_1 as u128 * r2 as u128) / (r1 as u128)) as u64;
                // The amount of asset 1 we get from asset 2 at current prices.
                let lambda_1_netted = ((delta_2 as u128 * r1 as u128) / (r2 as u128)) as u64;

                match (lambda_1_netted <= delta_1, lambda_2_netted <= delta_2) {
                    // We have more delta_1 than is needed to net out delta_2.
                    (true, _) => (
                        lambda_1_netted,
                        self.trade_1_to_2(delta_1 - lambda_1_netted) + delta_2,
                    ),
                    // We have more delta_2 than is needed to net out delta_1.
                    (_, true) => (
                        self.trade_2_to_1(delta_2 - lambda_2_netted) + delta_1,
                        lambda_2_netted,
                    ),
                    // Intuitively, these should never happen -- but skipping
                    // handling them would require justifying why, so instead,
                    // just burn all the input funds (lol)
                    (false, false) => (0, 0),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpmm() {
        // test:
        //   inputs (100,100) reserves (1,1) outputs should be (100,100)
        let mut cpmm = StubCpmm {
            reserves: Reserves { r1: 1, r2: 1 },
        };

        assert_eq!(cpmm.trade_netted((100, 100)), (100, 100));
    }
}
