use std::num::NonZeroU32;

use anyhow::{ensure, Context};
use penumbra_sdk_keys::PositionMetadataKey;
use prost::Message;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use penumbra_sdk_asset::{balance, Balance, Value};
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};

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
    /// Contains encrypted metadata about the position.
    pub encrypted_metadata: EncryptedPositionMetadata,
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

/// A strategy that a bundle of positions can adopt.
#[derive(Debug, Clone, PartialEq)]
pub enum Strategy {
    Skip,
    Arbitrary,
    Linear,
    Stable,
    Custom(u32),
}

/// Metadata about a position, including a strategy and identifier.
/// More detials can be found in UIP-9.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionMetadata {
    pub strategy: Strategy,
    pub identifier: NonZeroU32,
}

impl PositionMetadata {
    /// Encrypt the position metadata using the position metadata key,
    pub fn encrypt_with_random_nonce(self, pmk: &PositionMetadataKey) -> EncryptedPositionMetadata {
        use rand_core::RngCore;

        let mut raw_nonce = [0u8; 24];
        OsRng.fill_bytes(&mut raw_nonce);
        self.encrypt_with_nonce(pmk, raw_nonce)
    }

    pub fn encrypt_with_nonce(
        self,
        pmk: &PositionMetadataKey,
        nonce: [u8; 24],
    ) -> EncryptedPositionMetadata {
        let proto_metadata: pb::PositionMetadata = self.into();
        let raw_metadata = proto_metadata.encode_to_vec();

        let cipher = XChaCha20Poly1305::new(&pmk.0);
        let nonce = XNonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(nonce, raw_metadata.as_ref())
            .expect("encryption succeeds");

        let mut bytes = Vec::with_capacity(ENCRYPTED_POSMETA_LEN);
        bytes.extend_from_slice(nonce.as_ref());
        bytes.extend_from_slice(&ciphertext);

        EncryptedPositionMetadata { bytes }
    }
}

impl Default for PositionMetadata {
    fn default() -> Self {
        Self {
            strategy: Strategy::Skip,
            identifier: NonZeroU32::new(1).expect("identifier is nonzero"),
        }
    }
}

pub const ENCRYPTED_POSMETA_LEN: usize = 50;
pub const CLEAR_POSMETA_LEN: usize = 10;

#[derive(Clone, Debug)]
pub struct EncryptedPositionMetadata {
    /// The inner bytes can either have 0 or `ENCRYPTED_BACKREF_LEN` bytes.
    bytes: Vec<u8>,
}

impl EncryptedPositionMetadata {
    pub fn empty() -> Self {
        Self { bytes: vec![] }
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let encrypted_metadata_len = bytes.len();
        ensure!(
            encrypted_metadata_len == 0 || encrypted_metadata_len == ENCRYPTED_POSMETA_LEN,
            "encrypted metadata length mismatch (expected zero or {}, got {})",
            ENCRYPTED_POSMETA_LEN,
            encrypted_metadata_len
        );

        Ok(Self { bytes })
    }

    pub fn decrypt(self, pmk: &PositionMetadataKey) -> anyhow::Result<Option<PositionMetadata>> {
        if self.is_empty() {
            return Ok(None);
        }

        let cipher = XChaCha20Poly1305::new(&pmk.0);
        // The inner bytes are either empty or `ENCRYPTED_BACKREF_LEN` bytes.
        let nonce = XNonce::from_slice(&self.bytes[..24]);
        // This includes a postfix for the authentication tag.
        let ciphertext_with_auth_tag = &self.bytes[24..];

        let decrypted_metadata = cipher
            .decrypt(nonce, ciphertext_with_auth_tag)
            .map_err(|e| anyhow::anyhow!("failed to decrypt position metadata: {}", e))?;

        let pb_metadata = pb::PositionMetadata::decode(decrypted_metadata.as_ref())
            .context("failed to decode position metadata")?;
        let metadata: PositionMetadata = pb_metadata.try_into()?;
        Ok(Some(metadata))
    }
}

impl From<PositionMetadata> for pb::PositionMetadata {
    fn from(value: PositionMetadata) -> Self {
        Self {
            strategy: match value.strategy {
                Strategy::Skip => 1u32,
                Strategy::Arbitrary => 2u32,
                Strategy::Linear => 3u32,
                Strategy::Stable => 4u32,
                Strategy::Custom(v) => v,
            },
            identifier: value.identifier.into(),
        }
    }
}

impl TryFrom<pb::PositionMetadata> for PositionMetadata {
    type Error = anyhow::Error;

    fn try_from(v: pb::PositionMetadata) -> Result<Self, Self::Error> {
        ensure!(v.strategy > 0, "missing strategy tag");
        ensure!(v.identifier > 0, "missing identifier tag");

        let strategy = v.strategy;

        let strategy = match strategy {
            1 => Strategy::Skip,
            2 => Strategy::Arbitrary,
            3 => Strategy::Linear,
            4 => Strategy::Stable,
            custom => Strategy::Custom(custom),
        };

        let identifier = NonZeroU32::new(v.identifier).expect("identifier is nonzero");

        Ok(Self {
            strategy,
            identifier,
        })
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
            encrypted_metadata: value.encrypted_metadata.bytes.into(),
        }
    }
}

impl TryFrom<pb::PositionOpen> for PositionOpen {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionOpen) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value
                .position
                .ok_or_else(|| anyhow::anyhow!("missing position"))?
                .try_into()?,
            encrypted_metadata: EncryptedPositionMetadata::from_bytes(value.encrypted_metadata)?,
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

#[cfg(test)]
mod tests {
    use crate::lp::action::CLEAR_POSMETA_LEN;
    use crate::lp::action::ENCRYPTED_POSMETA_LEN;

    use super::pb;
    use super::PositionMetadata;
    use penumbra_sdk_keys::keys::Bip44Path;
    use penumbra_sdk_keys::keys::SeedPhrase;
    use penumbra_sdk_keys::keys::SpendKey;
    use prost::Message;
    use rand_core::OsRng;
    use std::num::NonZeroU32;
    use std::u32;

    #[test]
    fn encrypted_metadata_len() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let proto: pb::PositionMetadata = posmet.clone().into();
        let raw_metadata = proto.encode_to_vec();
        let size = raw_metadata.len();
        assert_eq!(size, CLEAR_POSMETA_LEN);

        let pmk = fvk_sender.position_metadata_key();
        let encrypted_posmet = posmet.encrypt_with_random_nonce(&pmk);
        let size = encrypted_posmet.bytes.len();
        assert_eq!(size, ENCRYPTED_POSMETA_LEN);
    }

    #[test]
    fn encrypted_format_check() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let pmk = fvk_sender.position_metadata_key();
        let raw_nonce = [0u8; 24];
        let encrypted_posmet = posmet.encrypt_with_nonce(&pmk, raw_nonce.clone());
        assert_eq!(encrypted_posmet.bytes.len(), ENCRYPTED_POSMETA_LEN);

        let nonce = encrypted_posmet.bytes[..24].to_vec();
        assert_eq!(nonce, raw_nonce);
    }

    #[test]
    fn encrypted_metadata_roundtrip() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let proto: pb::PositionMetadata = posmet.clone().into();
        let raw_metadata = proto.encode_to_vec();
        let size = raw_metadata.len();

        assert_eq!(size, CLEAR_POSMETA_LEN);

        let pmk = fvk_sender.position_metadata_key();
        let encrypted_posmet = posmet.clone().encrypt_with_random_nonce(&pmk);
        let size = encrypted_posmet.bytes.len();
        assert_eq!(size, ENCRYPTED_POSMETA_LEN);

        let decrypted_posmet = encrypted_posmet.decrypt(&pmk).unwrap().unwrap();
        assert!(decrypted_posmet == posmet);
    }

    #[test]
    fn fixed_wire_size_some_id() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, CLEAR_POSMETA_LEN);
    }

    #[test]
    fn fixed_wire_size_max_id() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(u32::MAX).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, CLEAR_POSMETA_LEN);
    }

    #[test]
    fn fixed_wire_size_max_strat_max_id() {
        let proto = pb::PositionMetadata {
            strategy: 127u32,
            identifier: u32::MAX,
        };
        let size = proto.encoded_len();
        assert_eq!(size, CLEAR_POSMETA_LEN);
    }

    #[test]
    fn fixed_wire_size_small_id() {
        let posmet = PositionMetadata {
            strategy: super::Strategy::Arbitrary,
            identifier: NonZeroU32::new(1u32).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, CLEAR_POSMETA_LEN);
    }

    #[test]
    #[should_panic]
    fn domain_type_invalid_identifier() {
        let proto = pb::PositionMetadata {
            strategy: 127,
            identifier: 0,
        };
        let _: PositionMetadata = proto.try_into().unwrap();
    }

    #[test]
    /// Tests that metadata passing through the domain type is not lossy
    /// and that the wire size is correct.
    fn domain_type_max_strategy() {
        let original_proto = pb::PositionMetadata {
            strategy: u32::MAX,
            identifier: u32::MAX,
        };
        let domain: PositionMetadata = original_proto.clone().try_into().unwrap();

        let expected = PositionMetadata {
            strategy: super::Strategy::Custom(u32::MAX),
            identifier: NonZeroU32::new(u32::MAX).unwrap(),
        };
        assert_eq!(domain, expected);

        let new_proto: pb::PositionMetadata = domain.clone().into();
        assert_eq!(new_proto, original_proto);

        let serialized = new_proto.encode_to_vec();
        assert_eq!(serialized.len(), CLEAR_POSMETA_LEN);
    }

    #[test]
    #[should_panic]
    fn domain_type_invalid_zero_strategy() {
        let proto = pb::PositionMetadata {
            strategy: 0,
            identifier: 1,
        };
        let _: PositionMetadata = proto.try_into().unwrap();
    }

    #[test]
    #[should_panic]
    fn domain_type_invalid_variant_invalid_id() {
        let proto = pb::PositionMetadata {
            strategy: 0,
            identifier: 0,
        };
        let _: PositionMetadata = proto.try_into().unwrap();
    }

    #[test]
    fn empty_is_empty() {
        let empty = super::EncryptedPositionMetadata::empty();
        assert!(empty.is_empty());
    }
}
