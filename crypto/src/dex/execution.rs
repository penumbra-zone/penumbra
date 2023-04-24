use crate::dex::lp::{BareTradingFunction, TradingFunction};
use crate::dex::trading_pair::DirectedTradingPair;
use crate::{asset, Value};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapExecution", into = "pb::SwapExecution")]
pub struct SwapExecution {
    pub traces: Vec<TradeTrace>,
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
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl From<SwapExecution> for pb::SwapExecution {
    fn from(se: SwapExecution) -> Self {
        pb::SwapExecution {
            traces: se.traces.into_iter().map(Into::into).collect(),
        }
    }
}

/// Contains all individual steps consisting of a trade trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::TradeTrace", into = "pb::TradeTrace")]
pub struct TradeTrace {
    pub steps: Vec<TradeTraceStep>,
}

impl DomainType for TradeTrace {
    type Proto = pb::TradeTrace;
}

impl TryFrom<pb::TradeTrace> for TradeTrace {
    type Error = anyhow::Error;
    fn try_from(tt: pb::TradeTrace) -> Result<Self> {
        Ok(Self {
            steps: tt
                .steps
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl From<TradeTrace> for pb::TradeTrace {
    fn from(se: TradeTrace) -> Self {
        pb::TradeTrace {
            steps: se.steps.into_iter().map(Into::into).collect(),
        }
    }
}

/// An individual step within a trade trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::TradeTraceStep", into = "pb::TradeTraceStep")]
pub struct TradeTraceStep {
    /// The input asset and amount.
    pub input: Value,
    /// The output asset and amount.
    pub output: Value,
}

impl DomainType for TradeTraceStep {
    type Proto = pb::TradeTraceStep;
}

impl TryFrom<pb::TradeTraceStep> for TradeTraceStep {
    type Error = anyhow::Error;
    fn try_from(tts: pb::TradeTraceStep) -> Result<Self> {
        Ok(Self {
            input: tts
                .input
                .ok_or_else(|| anyhow::anyhow!("missing input"))?
                .try_into()?,
            output: tts
                .output
                .ok_or_else(|| anyhow::anyhow!("missing output"))?
                .try_into()?,
        })
    }
}

impl From<TradeTraceStep> for pb::TradeTraceStep {
    fn from(tts: TradeTraceStep) -> Self {
        pb::TradeTraceStep {
            input: Some(tts.input.into()),
            output: Some(tts.output.into()),
        }
    }
}
