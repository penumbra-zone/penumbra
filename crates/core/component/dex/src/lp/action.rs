use serde::{Deserialize, Serialize};

use penumbra_sdk_asset::{balance, Balance, Value};
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};

use super::{position, position::Position, LpNft};

/// A transaction action that opens a new position.
///
/// This action's contribution to the transaction's value balance is to consume
/// the initial reserves and contribute an opened position NFT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionOpen", into = "pb::PositionOpen")]
pub struct PositionOpen {
    /// Contains the data defining the position, sufficient to compute its `PositionId`.
    ///
    /// Positions are immutable, so the `PositionData` (and hence the `PositionId`)
    /// are unchanged over the entire lifetime of the position.
    pub position: Position,
    /// Optional encrypted metadata about this position.
    ///
    /// If present, this is a 50-byte blob containing a 24-byte nonce and a 26-byte ciphertext
    /// (a 10-byte serialized `PositionMetadata` plus a 16-byte authentication tag).
    pub encrypted_metadata: Option<Vec<u8>>,
}

impl EffectingData for PositionOpen {
    fn effect_hash(&self) -> EffectHash {
        // The position open action consists only of the position, which
        // we consider effecting data.
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl PositionOpen {
    /// Compute a commitment to the value this action contributes to its transaction.
    pub fn balance(&self) -> Balance {
        let opened_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position.id(), position::State::Opened).asset_id(),
        };

        let reserves = self.position.reserves.balance(&self.position.phi.pair);

        // The action consumes the reserves and produces an LP NFT
        Balance::from(opened_position_nft) - reserves
    }
}

/// A transaction action that closes a position.
///
/// This action's contribution to the transaction's value balance is to consume
/// an opened position NFT and contribute a closed position NFT.
///
/// Closing a position does not immediately withdraw funds, because Penumbra
/// transactions (like any ZK transaction model) are early-binding: the prover
/// must know the state transition they prove knowledge of, and they cannot know
/// the final reserves with certainty until after the position has been deactivated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionClose", into = "pb::PositionClose")]
pub struct PositionClose {
    pub position_id: position::Id,
}

impl EffectingData for PositionClose {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl PositionClose {
    /// Compute the value this action contributes to its transaction.
    pub fn balance(&self) -> Balance {
        let opened_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Opened).asset_id(),
        };

        let closed_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Closed).asset_id(),
        };

        // The action consumes an opened position and produces a closed position.
        Balance::from(closed_position_nft) - opened_position_nft
    }
}

/// A transaction action that withdraws funds from a closed position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// closed position NFT and contribute a withdrawn position NFT, as well as all
/// of the funds that were in the position at the time of closing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionWithdraw", into = "pb::PositionWithdraw")]
pub struct PositionWithdraw {
    pub position_id: position::Id,
    /// A transparent (zero blinding factor) commitment to the position's final reserves and fees.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    pub reserves_commitment: balance::Commitment,
    /// The sequence number of the withdrawal, allowing multiple withdrawals from the same position.
    pub sequence: u64,
}

impl EffectingData for PositionWithdraw {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for PositionOpen {
    type Proto = pb::PositionOpen;
}

impl From<PositionOpen> for pb::PositionOpen {
    fn from(value: PositionOpen) -> Self {
        Self {
            position: Some(value.position.into()),
            encrypted_metadata: value.encrypted_metadata.unwrap_or_default(),
        }
    }
}

impl TryFrom<pb::PositionOpen> for PositionOpen {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionOpen) -> Result<Self, Self::Error> {
        // Validate that if encrypted_metadata is present, it's exactly 50 bytes (24-byte nonce + 26-byte ciphertext)
        if !value.encrypted_metadata.is_empty() && value.encrypted_metadata.len() != 50 {
            anyhow::bail!("encrypted_metadata must be exactly 50 bytes if present");
        }

        Ok(Self {
            position: value
                .position
                .ok_or_else(|| anyhow::anyhow!("missing position"))?
                .try_into()?,
            encrypted_metadata: if value.encrypted_metadata.is_empty() {
                None
            } else {
                Some(value.encrypted_metadata)
            },
        })
    }
}

impl DomainType for PositionClose {
    type Proto = pb::PositionClose;
}

impl From<PositionClose> for pb::PositionClose {
    fn from(value: PositionClose) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
        }
    }
}

impl TryFrom<pb::PositionClose> for PositionClose {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionClose) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: value
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
        })
    }
}

impl DomainType for PositionWithdraw {
    type Proto = pb::PositionWithdraw;
}

impl From<PositionWithdraw> for pb::PositionWithdraw {
    fn from(value: PositionWithdraw) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            reserves_commitment: Some(value.reserves_commitment.into()),
            sequence: value.sequence,
        }
    }
}

impl TryFrom<pb::PositionWithdraw> for PositionWithdraw {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionWithdraw) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: value
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
            reserves_commitment: value
                .reserves_commitment
                .ok_or_else(|| anyhow::anyhow!("missing balance_commitment"))?
                .try_into()?,
            sequence: value.sequence,
        })
    }
}
