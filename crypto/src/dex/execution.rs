use crate::dex::lp::{BareTradingFunction, TradingFunction};
use crate::dex::trading_pair::DirectedTradingPair;
use crate::{asset, Value};
use anyhow::Result;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

/// Contains a path for a trade, including the trading pair (with direction), the trading
/// function defining their relationship, and the route taken between the two assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::Path", into = "pb::Path")]
pub struct Path {
    pair: DirectedTradingPair,
    route: Vec<asset::Id>,
    phi: BareTradingFunction,
}

impl Path {
    pub fn new(start: asset::Id, end: asset::Id, amm: TradingFunction) -> Result<Self> {
        Ok(Self {
            pair: DirectedTradingPair::new(start, end),
            route: vec![start, end],
            phi: amm.component,
        })
    }

    /// Extend the current path with the specified pool.
    pub fn extend(&mut self, pool: TradingFunction) {
        let end = if self.pair.end == pool.pair.asset_1() {
            pool.pair.asset_2()
        } else {
            pool.pair.asset_1()
        };

        let pair = DirectedTradingPair::new(self.start(), end.clone());
        let composed_amm = self.phi.compose(pool.component);
        self.route.push(end);
        self.pair = pair;
        self.phi = composed_amm;
    }

    pub fn start(&self) -> asset::Id {
        self.pair.start
    }

    pub fn end(&self) -> asset::Id {
        self.pair.end
    }
}

impl TypeUrl for Path {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.Path";
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

/// Contains the summary data of a trade, for client consumption.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapExecution", into = "pb::SwapExecution")]
pub struct SwapExecution {
    pub traces: Vec<Vec<Value>>,
}

impl TypeUrl for SwapExecution {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.SwapExecution";
}

impl DomainType for SwapExecution {
    type Proto = pb::SwapExecution;
}

impl TryFrom<pb::SwapExecution> for SwapExecution {
    type Error = anyhow::Error;
    fn try_from(se: pb::SwapExecution) -> Result<Self> {
        Ok(Self {
            traces: se
                .traces
                .into_iter()
                .map(|vt| {
                    vt.value
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl From<SwapExecution> for pb::SwapExecution {
    fn from(se: SwapExecution) -> Self {
        pb::SwapExecution {
            traces: se
                .traces
                .into_iter()
                .map(|vt| pb::swap_execution::Trace {
                    value: vt.into_iter().map(Into::into).collect(),
                })
                .collect(),
        }
    }
}
