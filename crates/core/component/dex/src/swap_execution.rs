use anyhow::Result;
use penumbra_crypto::{Amount, Value};
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
    /// Create a new `SwapExecution` from the given traces.
    /// Sets input and output based on trace values.
    pub fn new(traces: Vec<Vec<Value>>) -> Self {
        // Input consists of the sum of the first value of each trace.
        let input = traces
            .iter()
            .map(|trace| trace.first().expect("empty trace").amount)
            .sum::<Amount>();
        // Output consists of the sum of the last value of each trace.
        let output = traces
            .iter()
            .map(|trace| trace.last().expect("empty trace").amount)
            .sum::<Amount>();

        let in_asset_id = traces
            .first()
            .expect("empty traces")
            .first()
            .expect("empty trace")
            .asset_id;
        let out_asset_id = traces
            .first()
            .expect("empty traces")
            .last()
            .expect("empty trace")
            .asset_id;
        Self {
            traces,
            input: Value {
                amount: input,
                asset_id: in_asset_id,
            },
            output: Value {
                amount: output,
                asset_id: out_asset_id,
            },
        }
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
