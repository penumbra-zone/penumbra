use penumbra_proto::{penumbra::core::txhash::v1alpha1 as pb, DomainType, Message, Name};

/// A hash of a transaction's _effecting data_, describing its effects on the
/// chain state.
///
/// This includes, e.g., the commitments to new output notes created by the
/// transaction, or nullifiers spent by the transaction, but does not include
/// _authorizing data_ such as signatures or zk proofs.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EffectHash(pub [u8; 64]);

/// A helper function to create a BLAKE2b `State` instance given a variable-length personalization string.
fn create_personalized_state(personalization: &str) -> blake2b_simd::State {
    let mut state = blake2b_simd::State::new();

    // The `TypeUrl` provided as a personalization string is variable length,
    // so we first include the length in bytes as a fixed-length prefix.
    let length = personalization.len() as u64;
    state.update(&length.to_le_bytes());
    state.update(personalization.as_bytes());

    state
}

impl EffectHash {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// A helper function to hash the data of a proto-encoded message, using
    /// the variable-length `TypeUrl` of the corresponding domain type as a
    /// personalization string.
    pub fn from_proto_effecting_data<M: Message + Name>(message: &M) -> EffectHash {
        let mut state = create_personalized_state(&M::type_url());
        state.update(&message.encode_to_vec());

        EffectHash(*state.finalize().as_array())
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

impl DomainType for EffectHash {
    type Proto = pb::EffectHash;
}

impl From<EffectHash> for pb::EffectHash {
    fn from(msg: EffectHash) -> Self {
        Self {
            inner: msg.0.to_vec(),
        }
    }
}

impl TryFrom<pb::EffectHash> for EffectHash {
    type Error = anyhow::Error;
    fn try_from(value: pb::EffectHash) -> Result<Self, Self::Error> {
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
