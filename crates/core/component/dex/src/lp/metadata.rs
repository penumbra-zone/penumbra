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
