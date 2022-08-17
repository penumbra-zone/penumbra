use crate::symmetric::OutgoingCipherKey;
use crate::transaction::Fee;
use crate::{ka, Address};
use anyhow::{anyhow, Error, Result};
use penumbra_proto::{crypto as pb_crypto, dex as pb, Protobuf};

use crate::dex::TradingPair;
use crate::{
    keys::OutgoingViewingKey,
    note,
    symmetric::{PayloadKey, PayloadKind},
    value,
};

use super::{SwapCiphertext, OVK_WRAPPED_LEN_BYTES, SWAP_CIPHERTEXT_BYTES, SWAP_LEN_BYTES};

#[derive(Clone)]
pub struct SwapPlaintext {
    // Trading pair for the swap
    pub trading_pair: TradingPair,
    // Input amount of asset 1
    pub delta_1: u64,
    // Input amount of asset 2
    pub delta_2: u64,
    // Fee
    pub fee: Fee,
    // Address to receive the Swap NFT and SwapClaim outputs
    pub claim_address: Address,
}

impl SwapPlaintext {
    pub fn diversified_generator(&self) -> &decaf377::Element {
        self.claim_address.diversified_generator()
    }

    pub fn transmission_key(&self) -> &ka::Public {
        self.claim_address.transmission_key()
    }

    /// Generate encrypted outgoing cipher key for use with this swap.
    pub fn encrypt_key(
        &self,
        esk: &ka::Secret,
        ovk: &OutgoingViewingKey,
        cv: value::Commitment,
        cm: note::Commitment,
    ) -> [u8; OVK_WRAPPED_LEN_BYTES] {
        let epk = esk.diversified_public(self.diversified_generator());
        let ock = OutgoingCipherKey::derive(ovk, cv, cm, &epk);

        let mut op = Vec::new();
        op.extend_from_slice(&self.transmission_key().0);
        op.extend_from_slice(&esk.to_bytes());

        let encryption_result = ock.encrypt(op, PayloadKind::Swap);

        let wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES] = encryption_result
            .try_into()
            .expect("OVK encryption result fits in ciphertext len");

        wrapped_ovk
    }

    pub fn encrypt(&self, esk: &ka::Secret) -> SwapCiphertext {
        let epk = esk.diversified_public(self.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(self.transmission_key())
            .expect("key agreement succeeds");

        let key = PayloadKey::derive(&shared_secret, &epk);
        let swap_plaintext: [u8; SWAP_LEN_BYTES] = self.into();
        let encryption_result = key.encrypt(swap_plaintext.to_vec(), PayloadKind::Swap);

        let ciphertext: [u8; SWAP_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("swap encryption result fits in ciphertext len");

        SwapCiphertext(ciphertext)
    }

    pub fn from_parts(
        trading_pair: TradingPair,
        delta_1: u64,
        delta_2: u64,
        fee: Fee,
        claim_address: Address,
    ) -> Result<Self, Error> {
        Ok(SwapPlaintext {
            trading_pair,
            delta_1,
            delta_2,
            fee,
            claim_address,
        })
    }
}

impl Protobuf<pb::SwapPlaintext> for SwapPlaintext {}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        Ok(Self {
            delta_1: plaintext.delta_1,
            delta_2: plaintext.delta_2,
            claim_address: plaintext
                .claim_address
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext claim address"))?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid claim address in SwapPlaintext"))?,
            fee: Fee(plaintext
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext fee"))?
                .amount),
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
            delta_1: plaintext.delta_1,
            delta_2: plaintext.delta_2,
            fee: Some(pb_crypto::Fee {
                amount: plaintext.fee.0,
            }),
            claim_address: Some(plaintext.claim_address.into()),
            trading_pair: Some(plaintext.trading_pair.into()),
        }
    }
}

impl From<&SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: &SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        let mut bytes = [0u8; SWAP_LEN_BYTES];
        bytes[0..64].copy_from_slice(&swap.trading_pair.to_bytes());
        bytes[64..72].copy_from_slice(&swap.delta_1.to_le_bytes());
        bytes[72..80].copy_from_slice(&swap.delta_2.to_le_bytes());
        bytes[80..88].copy_from_slice(&swap.fee.0.to_le_bytes());
        let pb_address = pb_crypto::Address::from(swap.claim_address);
        bytes[88..168].copy_from_slice(&pb_address.inner);
        bytes
    }
}

impl From<SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        (&swap).into()
    }
}

impl TryFrom<&[u8]> for SwapPlaintext {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != SWAP_LEN_BYTES {
            return Err(anyhow!("incorrect length for serialized swap plaintext"));
        }

        let tp_bytes: [u8; 64] = bytes[0..64]
            .try_into()
            .map_err(|_| anyhow!("error fetching trading pair bytes"))?;
        let delta_1_bytes: [u8; 8] = bytes[64..72]
            .try_into()
            .map_err(|_| anyhow!("error fetching delta1 bytes"))?;
        let delta_2_bytes: [u8; 8] = bytes[72..80]
            .try_into()
            .map_err(|_| anyhow!("error fetching delta2 bytes"))?;
        let fee_bytes: [u8; 8] = bytes[80..88]
            .try_into()
            .map_err(|_| anyhow!("error fetching fee bytes"))?;
        let address_bytes: [u8; 80] = bytes[88..168]
            .try_into()
            .map_err(|_| anyhow!("error fetching address bytes"))?;
        let pb_address = pb_crypto::Address {
            inner: address_bytes.to_vec(),
        };

        SwapPlaintext::from_parts(
            tp_bytes
                .try_into()
                .map_err(|_| anyhow!("error deserializing trading pair"))?,
            u64::from_le_bytes(delta_1_bytes),
            u64::from_le_bytes(delta_2_bytes),
            Fee(u64::from_le_bytes(fee_bytes)),
            pb_address.try_into()?,
        )
    }
}

impl TryFrom<[u8; SWAP_LEN_BYTES]> for SwapPlaintext {
    type Error = Error;

    fn try_from(bytes: [u8; SWAP_LEN_BYTES]) -> Result<SwapPlaintext, Self::Error> {
        (&bytes[..]).try_into()
    }
}
