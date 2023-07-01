use penumbra_proto::{core::crypto::v1alpha1 as pb_crypto, DomainType, TypeUrl};
use penumbra_tct as tct;

/// Something that can be hashed to produce an [`EffectHash`].
pub trait EffectingData {
    fn effect_hash(&self) -> EffectHash;
}

/// Stateless verification context for a transaction.
///
/// TODO: this is located in this crate just for convenience (at the bottom of the dep tree).
#[derive(Clone, Debug)]
pub struct TransactionContext {
    /// The transaction's anchor.
    pub anchor: tct::Root,
    /// The transaction's effect hash.
    pub effect_hash: EffectHash,
}

/// A hash of a transaction's _effecting data_, describing its effects on the
/// chain state.
///
/// This includes, e.g., the commitments to new output notes created by the
/// transaction, or nullifiers spent by the transaction, but does not include
/// _authorizing data_ such as signatures or zk proofs.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EffectHash(pub [u8; 64]);

impl EffectHash {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Default for EffectHash {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl std::fmt::Debug for EffectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EffectHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl TypeUrl for EffectHash {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.EffectHash";
}

impl DomainType for EffectHash {
    type Proto = pb_crypto::EffectHash;
}

impl From<EffectHash> for pb_crypto::EffectHash {
    fn from(msg: EffectHash) -> Self {
        Self {
            inner: msg.0.to_vec(),
        }
    }
}

impl TryFrom<pb_crypto::EffectHash> for EffectHash {
    type Error = anyhow::Error;
    fn try_from(value: pb_crypto::EffectHash) -> Result<Self, Self::Error> {
        Ok(Self(value.inner.try_into().map_err(|_| {
            anyhow::anyhow!("incorrect length for effect hash")
        })?))
    }
}

impl AsRef<[u8]> for EffectHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
