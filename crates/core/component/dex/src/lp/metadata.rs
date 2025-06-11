use std::num::NonZeroU32;

use anyhow::Context;
use penumbra_sdk_keys::{
    symmetric::{POSITION_METADATA_NONCE_SIZE_BYTES, POSITION_METADATA_SIZE_BYTES},
    PositionMetadataKey,
};
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};

/// Metadata about a position, or bundle of positions.
///
/// See [UIP-9](https://uips.penumbra.zone/uip-9.html) for more details.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct PositionMetadata {
    /// A strategy tag for the bundle.
    pub strategy: NonZeroU32,

    /// A unique identifier for the bundle this position belongs to.
    pub identifier: NonZeroU32,
}

impl PositionMetadata {
    pub fn encrypt(
        self,
        pmk: &PositionMetadataKey,
        nonce: &[u8; POSITION_METADATA_NONCE_SIZE_BYTES],
    ) -> Vec<u8> {
        let bytes = self.encode_to_vec();
        let plaintext: [u8; POSITION_METADATA_SIZE_BYTES] = bytes
            .try_into()
            .expect("PositionMetadata MUST always be exactly POSITION_METADATA_SIZE_BYTES long");
        pmk.encrypt(&plaintext, nonce)
    }

    pub fn decrypt(
        pmk: &PositionMetadataKey,
        ciphertext: Option<&[u8]>,
    ) -> anyhow::Result<Option<Self>> {
        let Some(ciphertext) = ciphertext else {
            return Ok(None);
        };
        if ciphertext.is_empty() {
            return Ok(None);
        }
        let Some(bytes) = pmk.decrypt(ciphertext) else {
            return Ok(None);
        };

        let metadata = PositionMetadata::decode(bytes.as_slice())
            .context("failed to decode PositionMetadata from decrypted bytes")?;

        Ok(Some(metadata))
    }
}

impl DomainType for PositionMetadata {
    type Proto = pb::PositionMetadata;
}

impl From<PositionMetadata> for pb::PositionMetadata {
    fn from(value: PositionMetadata) -> Self {
        Self {
            strategy: value.strategy.into(),
            identifier: value.identifier.into(),
        }
    }
}

impl TryFrom<pb::PositionMetadata> for PositionMetadata {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            strategy: value
                .strategy
                .try_into()
                .context("strategy should be non zero")?,
            identifier: value
                .identifier
                .try_into()
                .context("identifier should be non zero")?,
        })
    }
}

impl Default for PositionMetadata {
    fn default() -> Self {
        Self {
            strategy: NonZeroU32::new(1).expect("1 is non-zero"),
            identifier: NonZeroU32::new(1).expect("1 is non-zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lp::PositionMetadata;

    use super::pb;
    use penumbra_sdk_keys::keys::Bip44Path;
    use penumbra_sdk_keys::keys::SeedPhrase;
    use penumbra_sdk_keys::keys::SpendKey;
    use penumbra_sdk_keys::symmetric::ENCRYPTED_POSITION_METADATA_SIZE_BYTES;
    use penumbra_sdk_keys::symmetric::POSITION_METADATA_SIZE_BYTES;
    use penumbra_sdk_keys::PositionMetadataKey;
    use prost::Message;
    use rand_core::OsRng;
    use std::num::NonZeroU32;

    #[test]
    fn encrypted_metadata_len() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(1337u32).unwrap(),
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let proto: pb::PositionMetadata = posmet.clone().into();
        let size = proto.encode_to_vec().len();
        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);

        let pmk = PositionMetadataKey::derive(fvk_sender.outgoing());
        let encrypted_posmet = posmet.encrypt(&pmk, &[0u8; 24]);
        let size = encrypted_posmet.len();
        assert_eq!(size, ENCRYPTED_POSITION_METADATA_SIZE_BYTES);
    }
    #[test]
    fn encrypted_format_check() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(1337u32).unwrap(),
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let pmk = fvk_sender.position_metadata_key();
        let raw_nonce = [0u8; 24];
        let encrypted_posmet = posmet.encrypt(&pmk, &raw_nonce.clone());
        assert_eq!(
            encrypted_posmet.len(),
            ENCRYPTED_POSITION_METADATA_SIZE_BYTES
        );

        let nonce = encrypted_posmet[..24].to_vec();
        assert_eq!(nonce, raw_nonce);
    }

    #[test]
    fn encrypted_metadata_roundtrip() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(55u32).unwrap(),
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();

        let proto: pb::PositionMetadata = posmet.clone().into();
        let raw_metadata = proto.encode_to_vec();
        let size = raw_metadata.len();

        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);

        let pmk = fvk_sender.position_metadata_key();
        let encrypted_posmet = posmet.clone().encrypt(&pmk, &[1u8; 24]);
        let size = encrypted_posmet.len();
        assert_eq!(size, ENCRYPTED_POSITION_METADATA_SIZE_BYTES);

        let decrypted_posmet = PositionMetadata::decrypt(&pmk, Some(&encrypted_posmet))
            .expect("decryption should succeed")
            .expect("decrypted metadata should not be None");
        assert!(decrypted_posmet == posmet);
    }

    #[test]
    fn fixed_wire_size_some_id() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(55u32).unwrap(),
            identifier: NonZeroU32::new(1337u32).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);
    }

    #[test]
    fn fixed_wire_size_max_id() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(u32::MAX).unwrap(),
            identifier: NonZeroU32::new(u32::MAX).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);
    }

    #[test]
    fn fixed_wire_size_max_strat_max_id() {
        let proto = pb::PositionMetadata {
            strategy: 127u32,
            identifier: u32::MAX,
        };
        let size = proto.encoded_len();
        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);
    }

    #[test]
    fn fixed_wire_size_small_id() {
        let posmet = PositionMetadata {
            strategy: NonZeroU32::new(1u32).unwrap(),
            identifier: NonZeroU32::new(1u32).unwrap(),
        };
        let proto: pb::PositionMetadata = posmet.into();
        let size = proto.encoded_len();
        assert_eq!(size, POSITION_METADATA_SIZE_BYTES);
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
            strategy: NonZeroU32::new(u32::MAX).unwrap(),
            identifier: NonZeroU32::new(u32::MAX).unwrap(),
        };
        assert_eq!(domain, expected);

        let new_proto: pb::PositionMetadata = domain.clone().into();
        assert_eq!(new_proto, original_proto);

        let serialized = new_proto.encode_to_vec();
        assert_eq!(serialized.len(), POSITION_METADATA_SIZE_BYTES);
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
    fn custom_strategy_lossy() {
        let metadata = PositionMetadata {
            strategy: NonZeroU32::new(1).unwrap(),
            identifier: NonZeroU32::new(1).unwrap(),
        };

        let proto: pb::PositionMetadata = metadata.clone().into();

        let roundtrip_metadata: PositionMetadata = proto.try_into().unwrap();
        assert_eq!(roundtrip_metadata, metadata);
    }
}
