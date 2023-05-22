use anyhow::Result;
use penumbra_crypto::Value;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

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
