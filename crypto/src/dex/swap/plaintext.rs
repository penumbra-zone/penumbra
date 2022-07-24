use crate::ka;
use crate::transaction::Fee;
use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use penumbra_proto::{dex as pb, Protobuf};

use crate::dex::TradingPair;
use crate::keys::OutgoingViewingKey;

use super::{
    Error, SwapCiphertext, OVK_WRAPPED_LEN_BYTES, SWAP_CIPHERTEXT_BYTES, SWAP_ENCRYPTION_NONCE,
    SWAP_LEN_BYTES, SWAP_TYPE,
};

#[derive(Clone)]
pub struct SwapPlaintext {
    // Trading pair for the swap
    pub trading_pair: TradingPair,
    // Amount of asset 1
    pub t1: u64,
    // Amount of asset 2
    pub t2: u64,
    // Fee
    pub fee: Fee,
    // Diversified basepoint
    pub b_d: decaf377::Element,
    // Diversified public key
    pub pk_d: ka::Public,
}

impl SwapPlaintext {
    // Create a new hash based on the ephemeral public key and shared secret suitable for use as a key for symmetric encryption.
    //
    // Implementing this way allows recovery of all swap plaintexts via the seed phrase.
    //
    // Theoretically, if a paranoid user did want to achieve forward secrecy, they could choose to encrypt
    // nonsense bytes as the swap plaintext as the swap ciphertext does not need to be valid for the
    // swap to succeed, however this is unsupported by the official client.
    pub(super) fn derive_symmetric_key(
        shared_secret: &ka::SharedSecret,
        epk: &ka::Public,
    ) -> blake2b_simd::Hash {
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&shared_secret.0);
        kdf.update(&epk.0);

        kdf.finalize()
    }

    pub fn diversified_generator(&self) -> decaf377::Element {
        self.b_d
    }

    pub fn transmission_key(&self) -> ka::Public {
        self.pk_d
    }

    /// Use Blake2b-256 to derive an encryption key `ock` from the OVK and public fields.
    pub fn derive_ock(ovk: &OutgoingViewingKey, epk: &ka::Public) -> blake2b_simd::Hash {
        // let cv_bytes: [u8; 32] = cv.into();
        // let cm_bytes: [u8; 32] = cm.into();

        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.0);
        // TODO: should we be using the public fields e.g. t1, t2, trading_pair here?
        // Note implementation uses value commitments...
        // kdf.update(&cv_bytes);
        // kdf.update(&cm_bytes);
        kdf.update(&epk.0);

        kdf.finalize()
    }

    /// Generate encrypted outgoing cipher key for use with this swap.
    pub fn encrypt_key(
        &self,
        esk: &ka::Secret,
        ovk: &OutgoingViewingKey,
    ) -> [u8; OVK_WRAPPED_LEN_BYTES] {
        let epk = esk.diversified_public(&self.diversified_generator());
        let kdf_output = SwapPlaintext::derive_ock(ovk, &epk);

        let ock = Key::from_slice(kdf_output.as_bytes());

        let mut op = Vec::new();
        op.extend_from_slice(&self.transmission_key().0);
        op.extend_from_slice(&esk.to_bytes());

        let cipher = ChaCha20Poly1305::new(ock);

        // Note: Here we use the same nonce as swap encryption, however the keys are different.
        // For swap encryption we derive a symmetric key from the shared secret and epk.
        // However, for encrypting the outgoing cipher key, we derive a symmetric key from the
        // sender's OVK, and the epk. Since the keys are
        // different, it is safe to use the same nonce.
        //
        // References:
        // * Section 5.4.3 of the ZCash protocol spec
        // * Section 2.3 RFC 7539
        let nonce = Nonce::from_slice(&*SWAP_ENCRYPTION_NONCE);

        let encryption_result = cipher
            .encrypt(nonce, op.as_ref())
            .expect("OVK encryption succeeded");

        let wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES] = encryption_result
            .try_into()
            .expect("OVK encryption result fits in ciphertext len");

        wrapped_ovk
    }

    pub fn encrypt(&self, esk: &ka::Secret) -> SwapCiphertext {
        let epk = esk.diversified_public(&self.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(&self.transmission_key())
            .expect("key agreement succeeds");

        let key = SwapPlaintext::derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*SWAP_ENCRYPTION_NONCE);

        let swap_plaintext: Vec<u8> = self.into();
        let encryption_result = cipher
            .encrypt(nonce, swap_plaintext.as_ref())
            .expect("swap encryption succeeded");

        let ciphertext: [u8; SWAP_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("swap encryption result fits in ciphertext len");

        SwapCiphertext(ciphertext)
    }

    pub fn from_parts(
        trading_pair: TradingPair,
        t1: u64,
        t2: u64,
        fee: Fee,
        b_d: decaf377::Element,
        pk_d: ka::Public,
    ) -> Result<Self, Error> {
        Ok(SwapPlaintext {
            trading_pair,
            t1,
            t2,
            fee,
            b_d,
            pk_d,
        })
    }
}

impl Protobuf<pb::SwapPlaintext> for SwapPlaintext {}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        let b_d_bytes: [u8; 32] = plaintext
            .b_d
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid diversified basepoint in SwapPlaintext"))?;
        let b_d_encoding = decaf377::Encoding(b_d_bytes);

        Ok(Self {
            t1: plaintext.t1,
            t2: plaintext.t2,
            fee: Fee(plaintext
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext fee"))?
                .amount),
            b_d: b_d_encoding.decompress().map_err(|_| {
                anyhow::anyhow!("error decompressing diversified basepoint in SwapPlaintext")
            })?,
            pk_d: ka::Public(
                plaintext.pk_d.try_into().map_err(|_| {
                    anyhow::anyhow!("invalid diversified publickey in SwapPlaintext")
                })?,
            ),
            trading_pair: plaintext
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing trading pair in SwapPlaintext"))?
                .try_into()?,
        })
    }
}

impl From<SwapPlaintext> for pb::SwapPlaintext {
    fn from(plaintext: SwapPlaintext) -> Self {
        Self {
            t1: plaintext.t1,
            t2: plaintext.t2,
            fee: Some(penumbra_proto::transaction::Fee {
                amount: plaintext.fee.0,
            }),
            b_d: plaintext.b_d.compress().0.to_vec(),
            pk_d: plaintext.pk_d.0.to_vec(),
            trading_pair: Some(plaintext.trading_pair.into()),
        }
    }
}

impl From<&SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: &SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        let mut bytes = [0u8; SWAP_LEN_BYTES];
        bytes[0] = SWAP_TYPE;
        bytes[1..65].copy_from_slice(&swap.trading_pair.to_bytes());
        bytes[65..73].copy_from_slice(&swap.t1.to_le_bytes());
        bytes[73..81].copy_from_slice(&swap.t2.to_le_bytes());
        bytes[81..89].copy_from_slice(&swap.fee.0.to_le_bytes());
        bytes[89..121].copy_from_slice(&swap.pk_d.0);
        bytes[121..153].copy_from_slice(&swap.b_d.compress().0);
        bytes
    }
}

impl From<SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        (&swap).into()
    }
}

impl From<&SwapPlaintext> for Vec<u8> {
    fn from(swap: &SwapPlaintext) -> Vec<u8> {
        let mut bytes = vec![SWAP_TYPE];
        bytes.extend_from_slice(&swap.trading_pair.to_bytes());
        bytes.extend_from_slice(&swap.t1.to_le_bytes());
        bytes.extend_from_slice(&swap.t2.to_le_bytes());
        bytes.extend_from_slice(&swap.fee.0.to_le_bytes());
        bytes.extend_from_slice(&swap.pk_d.0);
        bytes.extend_from_slice(&swap.b_d.compress().0);
        bytes
    }
}

impl TryFrom<&[u8]> for SwapPlaintext {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != SWAP_LEN_BYTES {
            return Err(Error::SwapDeserializationError);
        }

        if bytes[0] != SWAP_TYPE {
            return Err(Error::SwapTypeUnsupported);
        }

        let tp_bytes: [u8; 64] = bytes[1..65]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let t1_bytes: [u8; 8] = bytes[65..73]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let t2_bytes: [u8; 8] = bytes[73..81]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let fee_bytes: [u8; 8] = bytes[81..89]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let pk_d_bytes: [u8; 32] = bytes[89..121]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let b_d_bytes: [u8; 32] = bytes[121..153]
            .try_into()
            .map_err(|_| Error::SwapDeserializationError)?;
        let b_d_encoding = decaf377::Encoding(b_d_bytes);

        SwapPlaintext::from_parts(
            tp_bytes
                .try_into()
                .map_err(|_| Error::SwapDeserializationError)?,
            u64::from_le_bytes(t1_bytes),
            u64::from_le_bytes(t2_bytes),
            Fee(u64::from_le_bytes(fee_bytes)),
            b_d_encoding
                .decompress()
                .map_err(|_| Error::SwapDeserializationError)?,
            ka::Public(pk_d_bytes),
        )
    }
}

impl TryFrom<[u8; SWAP_LEN_BYTES]> for SwapPlaintext {
    type Error = Error;

    fn try_from(bytes: [u8; SWAP_LEN_BYTES]) -> Result<SwapPlaintext, Self::Error> {
        (&bytes[..]).try_into()
    }
}
