use crate::asset;
use crate::dex::lp::TradingFunction;
use crate::dex::trading_pair::DirectedTradingPair;
use anyhow::Result;

pub struct Path {
    pair: DirectedTradingPair,
    route: Vec<asset::Id>,
    phi: TradingFunction,
}

impl Path {
    pub fn new(start: asset::Id, end: asset::Id, amm: TradingFunction) -> Result<Self> {
        Ok(Self {
            pair: DirectedTradingPair::new(start, end),
            route: vec![start, end],
            phi: amm,
        })
    }

    pub fn extend(&mut self, psi: TradingFunction) {
        let end = if self.pair.end == psi.pair.asset_1() {
            psi.pair.asset_2()
        } else {
            psi.pair.asset_1()
        };

        let pair = DirectedTradingPair::new(self.start(), end.clone());
        let amm = self.phi.compose(psi, pair.into()).unwrap();
        self.route.push(end);
        self.pair = pair;
        self.phi = amm;
    }

    pub fn start(&self) -> asset::Id {
        self.pair.start
    }

    pub fn end(&self) -> asset::Id {
        self.pair.end
    }
}
