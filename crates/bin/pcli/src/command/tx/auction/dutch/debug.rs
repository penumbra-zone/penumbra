use crate::command::tx::auction::dutch::gda::GradualAuction;
use penumbra_auction::auction::dutch::DutchAuctionDescription;
use serde::Serialize;

/// A minimal representation of a DA description used for visualization/debugging
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

/// A minimal representation of a GDA used for visualization/debugging
#[derive(Debug, Clone, Serialize)]
pub struct DebugGda {
    pub input: u128,
    pub min_output: u128,
    pub max_output: u128,
    pub recipe: String,
    pub gda_start_height: u64,
    pub gda_end_height: u64,
}

impl From<GradualAuction> for DebugGda {
    fn from(gda: GradualAuction) -> Self {
        DebugGda {
            input: gda.input.amount.value(),
            min_output: gda.min_output.amount.value(),
            max_output: gda.max_output.amount.value(),
            recipe: gda.recipe.to_string(),
            gda_start_height: gda.start_height,
            gda_end_height: gda.start_height + gda.recipe.as_blocks(),
        }
    }
}
