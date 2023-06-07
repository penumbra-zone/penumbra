use anyhow::Result;
use penumbra_crypto::{fixpoint::U128x128, Amount, Value};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

/// Contains the summary data of a trade, for client consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapExecution", into = "pb::SwapExecution")]
pub struct SwapExecution {
    pub traces: Vec<Vec<Value>>,
    pub input: Value,
    pub output: Value,
}

impl SwapExecution {
    pub fn max_price(&self) -> Result<Option<U128x128>> {
        let Some(aggregate_input )= self.aggregate_input() else {
            return Ok(None)
        };

        let Some(aggregate_output) = self.aggregate_output() else {
            return Ok(None)
        };

        let price = U128x128::ratio(aggregate_input, aggregate_output)?;

        Ok(Some(price))
    }

    fn aggregate_input(&self) -> Option<Amount> {
        self.traces
            .iter()
            .fold(Some(Amount::zero()), |acc, execution_trace| {
                acc.and_then(|acc_input| {
                    execution_trace
                        .first()
                        .map(|input| acc_input + input.amount)
                })
            })
    }

    fn aggregate_output(&self) -> Option<Amount> {
        self.traces
            .iter()
            .fold(Some(Amount::zero()), |acc, execution_trace| {
                acc.and_then(|acc_output| {
                    execution_trace
                        .last()
                        .map(|output| acc_output + output.amount)
                })
            })
    }
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
            input: se
                .input
                .ok_or_else(|| anyhow::anyhow!("missing input"))?
                .try_into()?,
            output: se
                .output
                .ok_or_else(|| anyhow::anyhow!("missing output"))?
                .try_into()?,
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
            input: Some(se.input.into()),
            output: Some(se.output.into()),
        }
    }
}
