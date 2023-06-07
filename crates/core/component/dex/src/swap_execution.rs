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
    /// Returns the price of the latest execution trace.
    pub fn max_price(&self) -> Result<Option<U128x128>> {
        let Some((input, output)) = self.traces.last().and_then(|trace| {
            let input = trace.first()?;
            let output = trace.last()?;
            Some((input, output))
        }) else {
            return Ok(None)
        };

        let price = U128x128::ratio(input.amount, output.amount)?;
        Ok(Some(price))
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
