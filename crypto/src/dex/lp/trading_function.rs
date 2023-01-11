use anyhow::anyhow;
use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{dex::TradingPair, Amount};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::TradingFunction", into = "pb::TradingFunction")]
pub struct TradingFunction {
    pub component: BareTradingFunction,
    pub pair: TradingPair,
}

impl TradingFunction {
    fn new(pair: TradingPair, fee: u32, asset_1: Amount, asset_2: Amount) -> Self {
        Self {
            pair,
            component: BareTradingFunction {
                fee,
                p: asset_1,
                q: asset_2,
            },
        }
    }

    // Compose two `TradingFunction` together.
    //
    // ### Errors
    //
    // This function errors if two `TradingFunction`s are asset disjoint, for example: A <> B + C <> D
    fn compose(&self, phi: TradingFunction) -> anyhow::Result<TradingFunction> {
        // Since each pair has a canonical ordering, we must consider different cases
        // indepedently to correctly assign coefficients of the trading functions:
        //
        // Starting with the pair 1 <> 2:
        //              Pair_1  +   Pair_2     =   Synth_Pair
        //      Case A: 1 <> 2  +  2   <> 3    =  1   <> 3
        //      Case B: 1 <> 2  +  0   <> 2    =  0   <> 1
        //      Case C: 1 <> 2  +  1.5 <> 2    =  1   <> 1.5
        //      Case D: 1 <> 2  +  1   <> 3    =  2   <> 3
        //      Case E: 1 <> 2  +  1   <> 1.5  =  1.5 <> 2
        //      Case F: 1 <> 2  +  0   <> 1    =  0   <> 2
        let fee = self.component.fee * phi.component.fee;
        // Case A: (1 <> 2) + (2 <> 3) = (1 <> 3)
        if self.pair.asset_2() == phi.pair.asset_1() {
            let pair = TradingPair::new(self.pair.asset_1(), phi.pair.asset_2());
            let asset_1 = self.component.p * phi.component.p;
            let asset_2 = self.component.q * phi.component.q;
            let composed_amm = TradingFunction::new(pair, fee, asset_1, asset_2);
            Ok(composed_amm)
        } else if self.pair.asset_2() == phi.pair.asset_2() {
            let pair = TradingPair::new(self.pair.asset_1(), phi.pair.asset_1());

            // Case B: (1 <> 2) + (0 <> 2) = (0 <> 1)
            let mut asset_1 = self.component.q * phi.component.p;
            let mut asset_2 = self.component.p * phi.component.q;

            // Case C: (1 <> 2) + (1.5 <> 2) = (1 <> 1.5)
            if self.pair.asset_1() < phi.pair.asset_1() {
                std::mem::swap(&mut asset_1, &mut asset_2);
            }

            let composed_amm = TradingFunction::new(pair, fee, asset_1, asset_2);
            Ok(composed_amm)
        } else if self.pair.asset_1() == phi.pair.asset_1() {
            let pair = TradingPair::new(self.pair.asset_2(), phi.pair.asset_2());
            // Case D: (1 <> 2) + (1 <> 3) = (2 <> 3)
            let mut asset_1 = self.component.q * phi.component.p;
            let mut asset_2 = self.component.p * phi.component.q;

            // Case E: (1 <> 2) + (1 <> 1.5) = (1.5 <> 2)
            if self.pair.asset_2() > phi.pair.asset_2() {
                std::mem::swap(&mut asset_1, &mut asset_2);
            }

            let composed_amm = TradingFunction::new(pair, fee, asset_1, asset_2);
            Ok(composed_amm)
        } else if self.pair.asset_1() == phi.pair.asset_2() {
            // Case F: (1 <> 2) + (0 <> 1) = (0 <> 2)
            let pair = TradingPair::new(self.pair.asset_2(), phi.pair.asset_1());
            let asset_1 = phi.component.p * self.component.p;
            let asset_2 = phi.component.q * self.component.q;
            let composed_amm = TradingFunction::new(pair, fee, asset_1, asset_2);
            Ok(composed_amm)
        } else {
            Err(anyhow!(
                "cannot compose two TradingFunction that are asset disjoint"
            ))
        }
    }
}

impl TryFrom<pb::TradingFunction> for TradingFunction {
    type Error = anyhow::Error;

    fn try_from(phi: pb::TradingFunction) -> Result<Self, Self::Error> {
        Ok(Self {
            component: phi
                .component
                .ok_or_else(|| anyhow::anyhow!("missing BareTradingFunction"))?
                .try_into()?,
            pair: phi
                .pair
                .ok_or_else(|| anyhow::anyhow!("missing TradingPair"))?
                .try_into()?,
        })
    }
}

impl From<TradingFunction> for pb::TradingFunction {
    fn from(phi: TradingFunction) -> Self {
        Self {
            component: Some(phi.component.into()),
            pair: Some(phi.pair.into()),
        }
    }
}

impl Protobuf<pb::TradingFunction> for TradingFunction {}

/// The data describing a trading function.
///
/// This implicitly treats the trading function as being between assets 1 and 2,
/// without specifying what those assets are, to avoid duplicating data (each
/// asset ID alone is twice the size of the trading function).
///
/// The trading function is `phi(R) = p*R_1 + q*R_2` where `p` and `q` specify the quantity
/// of each asset in the pool.
/// This is used as a CFMM with constant `k` and fee `fee` (gamma).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::BareTradingFunction", into = "pb::BareTradingFunction")]
pub struct BareTradingFunction {
    pub fee: u32,
    pub p: Amount,
    pub q: Amount,
}

impl Protobuf<pb::BareTradingFunction> for BareTradingFunction {}

impl TryFrom<pb::BareTradingFunction> for BareTradingFunction {
    type Error = anyhow::Error;

    fn try_from(value: pb::BareTradingFunction) -> Result<Self, Self::Error> {
        Ok(Self {
            fee: value.fee,
            p: value
                .p
                .ok_or_else(|| anyhow::anyhow!("missing p"))?
                .try_into()?,
            q: value
                .q
                .ok_or_else(|| anyhow::anyhow!("missing q"))?
                .try_into()?,
        })
    }
}

impl From<BareTradingFunction> for pb::BareTradingFunction {
    fn from(value: BareTradingFunction) -> Self {
        Self {
            fee: value.fee,
            p: Some(value.p.into()),
            q: Some(value.q.into()),
        }
    }
}
