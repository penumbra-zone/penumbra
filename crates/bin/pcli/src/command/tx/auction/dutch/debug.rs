use penumbra_sdk_auction::auction::dutch::DutchAuctionDescription;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DebugDescription {
    pub input: u128,
    pub min_output: u128,
    pub max_output: u128,
    pub start_height: u64,
    pub end_height: u64,
    pub step_count: u64,
}

impl From<DutchAuctionDescription> for DebugDescription {
    fn from(desc: DutchAuctionDescription) -> Self {
        DebugDescription {
            input: desc.input.amount.value(),
            min_output: desc.min_output.value(),
            max_output: desc.max_output.value(),
            start_height: desc.start_height,
            end_height: desc.end_height,
            step_count: desc.step_count,
        }
    }
}
