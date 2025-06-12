use crate::keys::{IncomingViewingKey, OutgoingViewingKey};
use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce, XChaCha20Poly1305, XNonce,
};
use decaf377_ka as ka;
use penumbra_sdk_asset::balance;
use penumbra_sdk_proto::core::keys::v1::{self as pb};
use penumbra_sdk_tct::StateCommitment;
use rand::{CryptoRng, RngCore};

pub const PAYLOAD_KEY_LEN_BYTES: usize = 32;
pub const OVK_WRAPPED_LEN_BYTES: usize = 48;
pub const MEMOKEY_WRAPPED_LEN_BYTES: usize = 48;

/// Represents the item to be encrypted/decrypted with the [`PayloadKey`].
pub enum PayloadKind {
    /// Note is action-scoped.
    Note,
    /// MemoKey is action-scoped.
    MemoKey,
    /// Memo is transaction-scoped.
    Memo,
    /// Swap is action-scoped.
    Swap,
}

impl PayloadKind {
    pub(crate) fn nonce(&self) -> [u8; 12] {
        match self {
            Self::Note => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Self::MemoKey => [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Self::Swap => [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Self::Memo => [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting notes, swaps, memos, and memo keys.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PayloadKey(Key);

impl PayloadKey {
    /// Use Blake2b-256 to derive a `PayloadKey`.
    pub fn derive(shared_secret: &ka::SharedSecret, epk: &ka::Public) -> Self {
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.personal(b"Penumbra_Payload");
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&shared_secret.0);
        kdf.update(&epk.0);

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Derive a random `PayloadKey`. Used for memo key wrapping.
    pub fn random_key<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);
        Self(*Key::from_slice(&key_bytes[..]))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Encrypt a note, memo, or memo key using the `PayloadKey`.
    pub fn encrypt(&self, plaintext: Vec<u8>, kind: PayloadKind) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded")
    }

    /// Decrypt a note, memo, or memo key using the `PayloadKey`.
    pub fn decrypt(&self, ciphertext: Vec<u8>, kind: PayloadKind) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(&self.0);

        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))
    }

    /// Use Blake2b-256 to derive an encryption key from the OVK and public fields for swaps.
    pub fn derive_swap(ovk: &OutgoingViewingKey, cm: StateCommitment) -> Self {
        let cm_bytes: [u8; 32] = cm.into();

        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.personal(b"Penumbra_Payswap");
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());
        kdf.update(&cm_bytes);

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypt a swap using the `PayloadKey`.
    pub fn encrypt_swap(&self, plaintext: Vec<u8>) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = PayloadKind::Swap.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded")
    }

    /// Decrypt a swap using the `PayloadKey`.
    pub fn decrypt_swap(&self, ciphertext: Vec<u8>) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(&self.0);

        let nonce_bytes = PayloadKind::Swap.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))
    }
}

impl TryFrom<&[u8]> for PayloadKey {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; PAYLOAD_KEY_LEN_BYTES] = slice
            .try_into()
            .map_err(|_| anyhow::anyhow!("PayloadKey incorrect len"))?;
        Ok(Self(*Key::from_slice(&bytes)))
    }
}

impl TryFrom<Vec<u8>> for PayloadKey {
    type Error = anyhow::Error;

    fn try_from(vector: Vec<u8>) -> Result<Self, Self::Error> {
        vector.as_slice().try_into()
    }
}

impl From<[u8; 32]> for PayloadKey {
    fn from(bytes: [u8; 32]) -> Self {
        Self(*Key::from_slice(&bytes))
    }
}

impl TryFrom<pb::PayloadKey> for PayloadKey {
    type Error = anyhow::Error;
    fn try_from(msg: pb::PayloadKey) -> Result<Self, Self::Error> {
        msg.inner.as_slice().try_into()
    }
}

impl From<PayloadKey> for pb::PayloadKey {
    fn from(msg: PayloadKey) -> Self {
        pb::PayloadKey {
            inner: msg.0.to_vec(),
        }
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting [`OvkWrappedKey`] material used to decrypt
/// outgoing notes, and memos.
pub struct OutgoingCipherKey(Key);

impl OutgoingCipherKey {
    /// Use Blake2b-256 to derive an encryption key `ock` from the OVK and public fields.
    pub fn derive(
        ovk: &OutgoingViewingKey,
        cv: balance::Commitment,
        cm: StateCommitment,
        epk: &ka::Public,
    ) -> Self {
        let cv_bytes: [u8; 32] = cv.into();
        let cm_bytes: [u8; 32] = cm.into();

        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        kdf_params.personal(b"Penumbra_OutCiph");
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());
        kdf.update(&cv_bytes);
        kdf.update(&cm_bytes);
        kdf.update(&epk.0);

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypt key material using the `OutgoingCipherKey`.
    pub fn encrypt(&self, plaintext: Vec<u8>, kind: PayloadKind) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new(&self.0);

        // Note: Here we use the same nonce as note encryption, however the keys are different.
        // For note encryption we derive the `PayloadKey` symmetric key from the shared secret and epk.
        // However, for the outgoing cipher key, we derive a symmetric key from the
        // sender's OVK, balance commitment, note commitment, and the epk. Since the keys are
        // different, it is safe to use the same nonce.
        //
        // References:
        // * Section 5.4.3 of the ZCash protocol spec
        // * Section 2.3 RFC 7539
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded")
    }

    /// Decrypt key material using the `OutgoingCipherKey`.
    pub fn decrypt(&self, ciphertext: Vec<u8>, kind: PayloadKind) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))
    }
}

/// Represents encrypted key material used to reconstruct a `PayloadKey`.
#[derive(Clone, Debug)]
pub struct OvkWrappedKey(pub [u8; OVK_WRAPPED_LEN_BYTES]);

impl OvkWrappedKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for OvkWrappedKey {
    type Error = anyhow::Error;

    fn try_from(vector: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes: [u8; OVK_WRAPPED_LEN_BYTES] = vector
            .try_into()
            .map_err(|_| anyhow::anyhow!("wrapped OVK malformed"))?;
        Ok(Self(bytes))
    }
}

impl TryFrom<&[u8]> for OvkWrappedKey {
    type Error = anyhow::Error;

    fn try_from(arr: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; OVK_WRAPPED_LEN_BYTES] = arr
            .try_into()
            .map_err(|_| anyhow::anyhow!("wrapped OVK malformed"))?;
        Ok(Self(bytes))
    }
}

/// Represents encrypted key material used to decrypt a `MemoCiphertext`.
#[derive(Clone, Debug)]
pub struct WrappedMemoKey(pub [u8; MEMOKEY_WRAPPED_LEN_BYTES]);

impl WrappedMemoKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Encrypt a memo key using the action-specific `PayloadKey`.
    pub fn encrypt(
        memo_key: &PayloadKey,
        esk: ka::Secret,
        transmission_key: &ka::Public,
        diversified_generator: &decaf377::Element,
    ) -> Self {
        // 1. Construct the per-action PayloadKey.
        let epk = esk.diversified_public(diversified_generator);
        let shared_secret = esk
            .key_agreement_with(transmission_key)
            .expect("key agreement succeeded");

        let action_key = PayloadKey::derive(&shared_secret, &epk);
        // 2. Now use the per-action key to encrypt the memo key.
        let encrypted_memo_key = action_key.encrypt(memo_key.to_vec(), PayloadKind::MemoKey);
        let wrapped_memo_key_bytes: [u8; MEMOKEY_WRAPPED_LEN_BYTES] = encrypted_memo_key
            .try_into()
            .expect("memo key must fit in wrapped memo key field");

        WrappedMemoKey(wrapped_memo_key_bytes)
    }

    /// Decrypt a wrapped memo key by first deriving the action-specific `PayloadKey`.
    pub fn decrypt(&self, epk: ka::Public, ivk: &IncomingViewingKey) -> Result<PayloadKey> {
        // 1. Construct the per-action PayloadKey.
        let shared_secret = ivk
            .key_agreement_with(&epk)
            .expect("key agreement succeeded");

        let action_key = PayloadKey::derive(&shared_secret, &epk);
        // 2. Now use the per-action key to decrypt the memo key.
        let decrypted_memo_key = action_key
            .decrypt(self.to_vec(), PayloadKind::MemoKey)
            .map_err(|_| anyhow!("decryption error"))?;

        decrypted_memo_key.try_into()
    }

    /// Decrypt a wrapped memo key using the action-specific `PayloadKey`.
    pub fn decrypt_outgoing(&self, action_key: &PayloadKey) -> Result<PayloadKey> {
        let decrypted_memo_key = action_key
            .decrypt(self.to_vec(), PayloadKind::MemoKey)
            .map_err(|_| anyhow!("decryption error"))?;
        decrypted_memo_key.try_into()
    }
}

impl TryFrom<Vec<u8>> for WrappedMemoKey {
    type Error = anyhow::Error;

    fn try_from(vector: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes: [u8; MEMOKEY_WRAPPED_LEN_BYTES] = vector
            .try_into()
            .map_err(|_| anyhow::anyhow!("wrapped memo key malformed"))?;
        Ok(Self(bytes))
    }
}

impl TryFrom<&[u8]> for WrappedMemoKey {
    type Error = anyhow::Error;

    fn try_from(arr: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; MEMOKEY_WRAPPED_LEN_BYTES] = arr
            .try_into()
            .map_err(|_| anyhow::anyhow!("wrapped memo key malformed"))?;
        Ok(Self(bytes))
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key used for Spend backreferences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BackreferenceKey(pub Key);

impl BackreferenceKey {
    pub fn derive(ovk: &OutgoingViewingKey) -> Self {
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.personal(b"Penumbra_Backref");
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }
}

pub const POSITION_METADATA_SIZE_BYTES: usize = 10;
pub const POSITION_METADATA_NONCE_SIZE_BYTES: usize = 24;
pub const POSITION_METADATA_AUTH_TAG_SIZE_BYTES: usize = 16;
pub const ENCRYPTED_POSITION_METADATA_SIZE_BYTES: usize = POSITION_METADATA_SIZE_BYTES
    + POSITION_METADATA_NONCE_SIZE_BYTES
    + POSITION_METADATA_AUTH_TAG_SIZE_BYTES;

/// Represents a symmetric `XChaCha20Poly1305` key used for Position metadata encryption.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PositionMetadataKey(pub Key);

impl PositionMetadataKey {
    /// Derives a PositionMetadataKey from an OutgoingViewingKey.
    ///
    /// See [UIP-9](https://uips.penumbra.zone/uip-9.html) for more details.
    pub fn derive(ovk: &OutgoingViewingKey) -> Self {
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.personal(b"Penumbra_PosMeta");
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypts position metadata using the PositionMetadataKey.
    pub fn encrypt(
        &self,
        plaintext: &[u8; POSITION_METADATA_SIZE_BYTES],
        nonce: &[u8; POSITION_METADATA_NONCE_SIZE_BYTES],
    ) -> Vec<u8> {
        let cipher = XChaCha20Poly1305::new(&self.0);
        let nonce = XNonce::from_slice(nonce.as_ref());

        // Get the encrypted data with authentication tag
        // The auth tag adds 16 bytes.
        let mut ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded");

        // Prepend the nonce to the ciphertext
        let mut result = Vec::with_capacity(ENCRYPTED_POSITION_METADATA_SIZE_BYTES);
        result.extend_from_slice(nonce.as_slice());
        result.append(&mut ciphertext);

        result
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Option<Vec<u8>> {
        if ciphertext.len() != ENCRYPTED_POSITION_METADATA_SIZE_BYTES {
            return None;
        }

        // Extract the nonce (first 24 bytes)
        let nonce = XNonce::from_slice(&ciphertext[..POSITION_METADATA_NONCE_SIZE_BYTES]);

        // Extract the ciphertext (everything after the nonce)
        let encrypted_data = &ciphertext[POSITION_METADATA_NONCE_SIZE_BYTES..];

        // Create cipher with our key
        let cipher = XChaCha20Poly1305::new(&self.0);

        // Decrypt the data returning none if decryption fails
        cipher.decrypt(nonce, encrypted_data).ok()
    }
}

impl TryFrom<pb::PositionMetadataKey> for PositionMetadataKey {
    type Error = anyhow::Error;
    fn try_from(msg: pb::PositionMetadataKey) -> Result<Self, Self::Error> {
        let key_bytes: [u8; 32] = msg
            .inner
            .try_into()
            .map_err(|_| anyhow!("position metadata key must be 32 bytes"))?;
        Ok(Self(*Key::from_slice(&key_bytes)))
    }
}

impl From<PositionMetadataKey> for pb::PositionMetadataKey {
    fn from(msg: PositionMetadataKey) -> Self {
        pb::PositionMetadataKey {
            inner: msg.0.to_vec(),
        }
    }
}
