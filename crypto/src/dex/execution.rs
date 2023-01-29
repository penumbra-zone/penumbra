use crate::asset;
use crate::dex::lp::TradingFunction;
use crate::dex::trading_pair::DirectedTradingPair;
use anyhow::Result;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// Contains a path for a trade, including the trading pair (with direction), the trading
/// function defining their relationship, and the route taken between the two assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::Path", into = "pb::Path")]
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

impl DomainType for Path {
    type Proto = pb::Path;
}

impl TryFrom<pb::Path> for Path {
    type Error = anyhow::Error;
    fn try_from(path: pb::Path) -> Result<Self> {
        Ok(Self {
            pair: path
                .pair
                .ok_or_else(|| anyhow::anyhow!("missing path pair"))?
                .try_into()?,
            route: path
                .route
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>>>()?,
            phi: path
                .phi
                .ok_or_else(|| anyhow::anyhow!("missing path phi"))?
                .try_into()?,
        })
    }
}

impl From<Path> for pb::Path {
    fn from(path: Path) -> Self {
        pb::Path {
            pair: Some(path.pair.into()),
            route: path.route.into_iter().map(Into::into).collect(),
            phi: Some(path.phi.into()),
        }
    }
}
