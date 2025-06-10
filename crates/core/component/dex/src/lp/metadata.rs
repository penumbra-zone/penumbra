use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// Strategy types for position bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStrategy {
    /// Skip metadata tracking for this position.
    Skip,
    /// Arbitrary trading function.
    Arbitrary,
    /// Linear trading function.
    Linear,
    /// Stable trading function.
    Stable,
    /// Reserved for future or custom strategy types.
    Unknown,
}

impl From<u32> for PositionStrategy {
    fn from(value: u32) -> Self {
        match value {
            1 => PositionStrategy::Skip,
            2 => PositionStrategy::Arbitrary,
            3 => PositionStrategy::Linear,
            4 => PositionStrategy::Stable,
            _ => PositionStrategy::Unknown,
        }
    }
}

impl From<PositionStrategy> for u32 {
    fn from(strategy: PositionStrategy) -> Self {
        match strategy {
            PositionStrategy::Skip => 1,
            PositionStrategy::Arbitrary => 2,
            PositionStrategy::Linear => 3,
            PositionStrategy::Stable => 4,
            PositionStrategy::Unknown => u32::MAX, // Use u32::MAX for unknown strategies
        }
    }
}

/// Metadata about a position, or bundle of positions.
///
/// See [UIP-9](https://uips.penumbra.zone/uip-9.html) for more details.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PositionMetadata {
    /// A strategy tag for the bundle.
    pub strategy: PositionStrategy,

    /// A unique identifier for the bundle this position belongs to.
    pub identifier: u32,
}

impl DomainType for PositionMetadata {
    type Proto = pb::PositionMetadata;
}

impl From<PositionMetadata> for pb::PositionMetadata {
    fn from(value: PositionMetadata) -> Self {
        Self {
            strategy: value.strategy.into(),
            identifier: value.identifier,
        }
    }
}

impl From<pb::PositionMetadata> for PositionMetadata {
    fn from(value: pb::PositionMetadata) -> Self {
        Self {
            strategy: value.strategy.into(),
            identifier: value.identifier,
        }
    }
}
