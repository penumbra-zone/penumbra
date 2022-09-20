use crate::symmetric::OutgoingCipherKey;
use crate::transaction::Fee;
use crate::{asset, ka, Address, Value};
use anyhow::{anyhow, Error, Result};
use ark_ff::PrimeField;
use decaf377::Fq;
use penumbra_proto::{crypto as pb_crypto, dex as pb, Protobuf};
use poseidon377::{hash_4, hash_6};

use crate::dex::TradingPair;
use crate::{
    keys::OutgoingViewingKey,
    note,
    symmetric::{PayloadKey, PayloadKind},
    value,
};

use super::{
    SwapCiphertext, DOMAIN_SEPARATOR, OVK_WRAPPED_LEN_BYTES, SWAP_CIPHERTEXT_BYTES, SWAP_LEN_BYTES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwapPlaintext {
    // Trading pair for the swap
    pub trading_pair: TradingPair,
    // User's input amount of asset 1
    pub delta_1_i: u64,
    // User's input amount of asset 2
    pub delta_2_i: u64,
    // Prepaid fee to claim the swap
    pub claim_fee: Fee,
    // Address to receive the Swap NFT and SwapClaim outputs
    pub claim_address: Address,
}

impl SwapPlaintext {
    // Constructs the unique asset ID for a swap as a poseidon hash of the input data for the swap.
    //
    // https://protocol.penumbra.zone/main/zswap/swap.html#swap-actions
    pub fn asset_id(&self) -> asset::Id {
        let asset_id_hash = hash_6(
            &DOMAIN_SEPARATOR,
            (
                self.claim_fee.0.amount.into(),
                self.claim_fee.0.asset_id.0,
                self.claim_address
                    .diversified_generator()
                    .vartime_compress_to_field(),
                *self.claim_address.transmission_key_s(),
                Fq::from_le_bytes_mod_order(&self.claim_address.clue_key().0[..]),
                hash_4(
                    &DOMAIN_SEPARATOR,
                    (
                        self.trading_pair.asset_1().0,
                        self.trading_pair.asset_2().0,
                        self.delta_1_i.into(),
                        self.delta_2_i.into(),
                    ),
                ),
            ),
        );

        asset::Id(asset_id_hash)
    }

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
        delta_1_i: u64,
        delta_2_i: u64,
        claim_fee: Fee,
        claim_address: Address,
    ) -> Result<Self, Error> {
        Ok(SwapPlaintext {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee,
            claim_address,
        })
    }
}

impl Protobuf<pb::SwapPlaintext> for SwapPlaintext {}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        Ok(Self {
            delta_1_i: plaintext.delta_1_i,
            delta_2_i: plaintext.delta_2_i,
            claim_address: plaintext
                .claim_address
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext claim address"))?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid claim address in SwapPlaintext"))?,
            claim_fee: plaintext
                .claim_fee
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext claim_fee"))?
                .try_into()?,
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
            delta_1_i: plaintext.delta_1_i,
            delta_2_i: plaintext.delta_2_i,
            claim_fee: Some(plaintext.claim_fee.into()),
            claim_address: Some(plaintext.claim_address.into()),
            trading_pair: Some(plaintext.trading_pair.into()),
        }
    }
}

impl From<&SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: &SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        let mut bytes = [0u8; SWAP_LEN_BYTES];
        bytes[0..64].copy_from_slice(&swap.trading_pair.to_bytes());
        bytes[64..72].copy_from_slice(&swap.delta_1_i.to_le_bytes());
        bytes[72..80].copy_from_slice(&swap.delta_2_i.to_le_bytes());
        bytes[80..88].copy_from_slice(&swap.claim_fee.0.amount.to_le_bytes());
        bytes[88..120].copy_from_slice(&swap.claim_fee.0.asset_id.to_bytes());
        let pb_address = pb_crypto::Address::from(swap.claim_address);
        bytes[120..200].copy_from_slice(&pb_address.inner);
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
        let fee_amount_bytes: [u8; 8] = bytes[80..88]
            .try_into()
            .map_err(|_| anyhow!("error fetching fee amount bytes"))?;
        let fee_asset_id_bytes: [u8; 32] = bytes[88..120]
            .try_into()
            .map_err(|_| anyhow!("error fetching fee asset ID bytes"))?;
        let address_bytes: [u8; 80] = bytes[120..200]
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
            Fee(Value {
                amount: u64::from_le_bytes(fee_amount_bytes),
                asset_id: asset::Id::try_from(fee_asset_id_bytes)?,
            }),
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

#[cfg(test)]
mod tests {
    use rand_core::OsRng;

    use super::*;
    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey},
        Value,
    };

    #[test]
    fn swap_encryption_and_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());
        let trading_pair = TradingPair {
            asset_1: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            asset_2: asset::REGISTRY.parse_denom("nala").unwrap().id(),
        };

        let swap = SwapPlaintext {
            trading_pair,
            delta_1_i: 100000,
            delta_2_i: 1,
            claim_fee: Fee(Value {
                amount: 3,
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            }),
            claim_address: dest,
        };
        let esk = ka::Secret::new(&mut rng);

        let ciphertext = swap.encrypt(&esk);
        let diversified_basepoint = dest.diversified_generator();
        let transmission_key = swap.transmission_key();
        let plaintext =
            SwapCiphertext::decrypt(&ciphertext, &esk, transmission_key, diversified_basepoint)
                .expect("can decrypt swap");

        assert_eq!(plaintext, swap);
    }
}
