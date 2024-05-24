use anyhow::{anyhow, Result};
use decaf377_fmd::Precision;
use penumbra_proto::{
    core::component::shielded_pool::v1::{self as pb},
    DomainType,
};
use serde::{Deserialize, Serialize};

pub mod state_key;

/// How long users have to switch to updated parameters.
pub const FMD_GRACE_PERIOD_BLOCKS_DEFAULT: u64 = 1 << 4;
/// How often we update the params, in terms of the number of grace periods
pub const FMD_UPDATE_FREQUENCY_GRACE_PERIOD: u64 = 4;

pub fn should_update_fmd_params(fmd_grace_period_blocks: u64, height: u64) -> bool {
    height % (fmd_grace_period_blocks * FMD_UPDATE_FREQUENCY_GRACE_PERIOD) == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdParameters", into = "pb::FmdParameters")]
pub struct Parameters {
    /// FMD Precision.
    pub precision: Precision,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl DomainType for Parameters {
    type Proto = pb::FmdParameters;
}

impl TryFrom<pb::FmdParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FmdParameters) -> Result<Self> {
        Ok(Parameters {
            precision: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<Parameters> for pb::FmdParameters {
    fn from(params: Parameters) -> Self {
        pb::FmdParameters {
            precision_bits: params.precision.bits() as u32,
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            precision: Precision::default(),
            as_of_block_height: 1,
        }
    }
}

/// A struct holding params for the sliding window algorithm
#[derive(Clone, Debug, PartialEq, Eq)]
struct SlidingWindow {
    window_blocks: uint32,
    targeted_detections_per_window: uint32,
}

impl SlidingWindow {
    pub fn updated_fmd_params(
        &self,
        old: &Parameters,
        state: MetaParametersAlgorithmState,
        height: u64,
        clue_count_delta: (u64, u64),
    ) -> (Parameters, MetaParametersAlgorithmState) {
        // An edge case, which should act as a constant.
        if self.window_blocks == 0 {
            return old;
        }

        let new_clues_in_block = clue_count_delta.1.saturating_sub(clue_count_delta.1);

        let projected_clues_in_window = self.window_blocks * new_clues_in_block;

        // To receive the power of two *above* the targeted number of clues,
        // take the base 2 logarithm, round down, and use 1 for 0 clues
        let required_precision = if projected_clues_in_window == 0 {
            Precision::new(1u8).expect("1 is a valid precision")
        } else {
            let lg_projected_clues = 63 - projected_clues_in_window.leading_zeros();
            Precision::new(lg_projected_clues).unwrap_or(Precision::MAX)
        };
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaParametersAlgorithm {
    /// Use a fixed precision forever.
    Fixed(Precision),
    /// Use a sliding window
    SlidingWindow(SlidingWindow),
}

/// Meta parameters governing how FMD parameters change.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdMetaParameters", into = "pb::FmdMetaParameters")]
pub struct MetaParameters {
    pub fmd_grace_period_blocks: u64,
    pub algorithm: MetaParametersAlgorithm,
}

impl TryFrom<pb::FmdMetaParameters> for MetaParameters {
    type Error = anyhow::Error;

    fn try_from(value: pb::FmdMetaParameters) -> Result<Self> {
        let fmd_grace_period_blocks = value.fmd_grace_period_blocks;
        let algorithm = match value
            .algorithm
            .ok_or(anyhow!("FmdMetaParameters missing algorithm"))?
        {
            pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(p) => {
                MetaParametersAlgorithm::Fixed(Precision::new(p as u8)?)
            }
            pb::fmd_meta_parameters::Algorithm::SlidingWindow(x) => {}
        };
        Ok(MetaParameters {
            fmd_grace_period_blocks,
            algorithm,
        })
    }
}

impl From<MetaParameters> for pb::FmdMetaParameters {
    fn from(value: MetaParameters) -> Self {
        let algorithm = match value.algorithm {
            MetaParametersAlgorithm::Fixed(p) => {
                pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(p.bits().into())
            }
            MetaParametersAlgorithm::SlidingWindow(SlidingWindow {
                window_blocks,
                targeted_detections_per_window,
            }) => pb::fmd_meta_parameters::Algorithm::SlidingWindow(
                pb::fmd_meta_parameters::AlgorithmSlidingWindow {
                    window_blocks,
                    targeted_detections_per_window,
                },
            ),
        };
        pb::FmdMetaParameters {
            fmd_grace_period_blocks: value.fmd_grace_period_blocks,
            algorithm: Some(algorithm),
        }
    }
}

impl DomainType for MetaParameters {
    type Proto = pb::FmdMetaParameters;
}

impl Default for MetaParameters {
    fn default() -> Self {
        Self {
            fmd_grace_period_blocks: FMD_GRACE_PERIOD_BLOCKS_DEFAULT,
            algorithm: MetaParametersAlgorithm::Fixed(Precision::default()),
        }
    }
}

/// If any, the current state for the algorithm we're using.
///
/// This allows algorithms to hold arbitrary state. The algorithms need to be able
/// to start from having no state and function appropriately, which allows for good
/// backwards-compatability.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb::FmdMetaParametersAlgorithmState",
    into = "pb::FmdMetaParametersAlgorithmState"
)]
enum MetaParametersAlgorithmState {
    /// A catch-all case to allow us to explicitly handle not having state
    Nothing,
    /// The state for the fixed algorithm
    Fixed,
    /// The state for the sliding window algorithm.
    SlidingWindow {
        /// The approximate number of clues in the previous window.
        approximate_clue_count: u32,
    },
}

impl TryFrom<pb::FmdMetaParametersAlgorithmState> for MetaParametersAlgorithmState {
    type Error = anyhow::Error;

    fn try_from(value: pb::FmdMetaParametersAlgorithmState) -> Result<Self> {
        Ok(match value.state {
            Some(pb::fmd_meta_parameters_algorithm_state::State::Fixed(_)) => Self::Fixed,
            Some(pb::fmd_meta_parameters_algorithm_state::State::SlidingWindow(x)) => {
                Self::SlidingWindow {
                    approximate_clue_count: x.approximate_clue_count,
                }
            }
            None => Self::Nothing,
        })
    }
}

impl From<MetaParametersAlgorithmState> for pb::FmdMetaParametersAlgorithmState {
    fn from(value: MetaParametersAlgorithmState) -> Self {
        let state = match value {
            MetaParametersAlgorithmState::Nothing => None,
            MetaParametersAlgorithmState::Fixed => {
                Some(pb::fmd_meta_parameters_algorithm_state::State::Fixed(
                    pb::fmd_meta_parameters_algorithm_state::FixedState {},
                ))
            }
            MetaParametersAlgorithmState::SlidingWindow {
                approximate_clue_count,
            } => Some(
                pb::fmd_meta_parameters_algorithm_state::State::SlidingWindow(
                    pb::fmd_meta_parameters_algorithm_state::SlidingWindowState {
                        approximate_clue_count,
                    },
                ),
            ),
        };
        pb::FmdMetaParametersAlgorithmState { state }
    }
}

impl DomainType for MetaParametersAlgorithmState {
    type Proto = pb::FmdMetaParametersAlgorithmState;
}

impl Default for MetaParametersAlgorithmState {
    fn default() -> Self {
        Self::Nothing
    }
}

impl MetaParameters {
    pub fn updated_fmd_params(
        &self,
        old: &Parameters,
        state: MetaParametersAlgorithmState,
        height: u64,
        clue_count_delta: (u64, u64),
    ) -> (Parameters, MetaParametersAlgorithmState) {
        if clue_count_delta.1 < clue_count_delta.0 {
            tracing::warn!(
                "decreasing clue count at height {}: {} then {}",
                height,
                clue_count_delta.0,
                clue_count_delta.1
            );
        }
        match self.algorithm {
            MetaParametersAlgorithm::Fixed(precision) => (
                Parameters {
                    precision,
                    as_of_block_height: height,
                },
                MetaParametersAlgorithmState::Fixed,
            ),
            MetaParametersAlgorithm::SlidingWindow(w) => {
                w.updated_fmd_params(old, state, height, clue_count_delta)
            }
        }
    }
}
