use anyhow::{anyhow, Error, Result};

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::r1cs::FqVar;
use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_fee::Fee;
use penumbra_proto::{
    core::keys::v1 as pb_keys, penumbra::core::component::dex::v1 as pb, DomainType,
};
use penumbra_tct::StateCommitment;
use poseidon377::{hash_1, hash_4, hash_7};
use rand_core::{CryptoRng, RngCore};

use decaf377_ka as ka;
use penumbra_asset::{asset, Value, ValueVar};
use penumbra_keys::{keys::OutgoingViewingKey, Address, AddressVar, PayloadKey};
use penumbra_num::{Amount, AmountVar};
use penumbra_shielded_pool::{Note, Rseed};
use penumbra_tct::r1cs::StateCommitmentVar;

use crate::{BatchSwapOutputData, TradingPair, TradingPairVar};

use super::{SwapCiphertext, SwapPayload, DOMAIN_SEPARATOR, SWAP_CIPHERTEXT_BYTES, SWAP_LEN_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwapPlaintext {
    // Trading pair for the swap
    pub trading_pair: TradingPair,
    // Input amount of asset 1
    pub delta_1_i: Amount,
    // Input amount of asset 2
    pub delta_2_i: Amount,
    // Prepaid fee to claim the swap
    pub claim_fee: Fee,
    // Address to receive the Swap NFT and SwapClaim outputs
    pub claim_address: Address,
    // Swap rseed
    pub rseed: Rseed,
}

pub static OUTPUT_1_BLINDING_DOMAIN_SEPARATOR: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.swapclaim.output1.blinding").as_bytes(),
    )
});
pub static OUTPUT_2_BLINDING_DOMAIN_SEPARATOR: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.swapclaim.output2.blinding").as_bytes(),
    )
});

impl SwapPlaintext {
    pub fn output_rseeds(&self) -> (Rseed, Rseed) {
        let fq_rseed = Fq::from_le_bytes_mod_order(&self.rseed.to_bytes()[..]);
        let rseed_1_hash = hash_1(&OUTPUT_1_BLINDING_DOMAIN_SEPARATOR, fq_rseed);
        let rseed_2_hash = hash_1(&OUTPUT_2_BLINDING_DOMAIN_SEPARATOR, fq_rseed);
        (
            Rseed(rseed_1_hash.to_bytes()),
            Rseed(rseed_2_hash.to_bytes()),
        )
    }

    pub fn output_notes(&self, batch_data: &BatchSwapOutputData) -> (Note, Note) {
        let (output_1_rseed, output_2_rseed) = self.output_rseeds();

        let (lambda_1_i, lambda_2_i) =
            batch_data.pro_rata_outputs((self.delta_1_i, self.delta_2_i));

        let output_1_note = Note::from_parts(
            self.claim_address,
            Value {
                amount: lambda_1_i,
                asset_id: self.trading_pair.asset_1(),
            },
            output_1_rseed,
        )
        .expect("claim address is valid");

        let output_2_note = Note::from_parts(
            self.claim_address,
            Value {
                amount: lambda_2_i,
                asset_id: self.trading_pair.asset_2(),
            },
            output_2_rseed,
        )
        .expect("claim address is valid");

        (output_1_note, output_2_note)
    }

    // Constructs the unique asset ID for a swap as a poseidon hash of the input data for the swap.
    //
    // https://protocol.penumbra.zone/main/zswap/swap.html#swap-actions
    pub fn swap_commitment(&self) -> StateCommitment {
        let inner = hash_7(
            &DOMAIN_SEPARATOR,
            (
                Fq::from_le_bytes_mod_order(&self.rseed.to_bytes()[..]),
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

        StateCommitment(inner)
    }

    pub fn diversified_generator(&self) -> &decaf377::Element {
        self.claim_address.diversified_generator()
    }

    pub fn transmission_key(&self) -> &ka::Public {
        self.claim_address.transmission_key()
    }

    pub fn encrypt(&self, ovk: &OutgoingViewingKey) -> SwapPayload {
        let commitment = self.swap_commitment();
        let key = PayloadKey::derive_swap(ovk, commitment);
        let swap_plaintext: [u8; SWAP_LEN_BYTES] = self.into();
        let encryption_result = key.encrypt_swap(swap_plaintext.to_vec(), commitment);

        let ciphertext: [u8; SWAP_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("swap encryption result fits in ciphertext len");

        SwapPayload {
            encrypted_swap: SwapCiphertext(ciphertext),
            commitment,
        }
    }

    pub fn delta_1_value(&self) -> Value {
        Value {
            amount: self.delta_1_i,
            asset_id: self.trading_pair.asset_1,
        }
    }

    pub fn delta_2_value(&self) -> Value {
        Value {
            amount: self.delta_2_i,
            asset_id: self.trading_pair.asset_2,
        }
    }

    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        trading_pair: TradingPair,
        delta_1_i: Amount,
        delta_2_i: Amount,
        claim_fee: Fee,
        claim_address: Address,
    ) -> SwapPlaintext {
        let rseed = Rseed::generate(rng);

        Self {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee,
            claim_address,
            rseed,
        }
    }
}

pub struct SwapPlaintextVar {
    pub claim_fee: ValueVar,
    pub delta_1_i: AmountVar,
    pub trading_pair: TradingPairVar,
    pub delta_2_i: AmountVar,
    pub claim_address: AddressVar,
    pub rseed: FqVar,
}

impl SwapPlaintextVar {
    pub fn delta_1_value(&self) -> ValueVar {
        ValueVar {
            amount: self.delta_1_i.clone(),
            asset_id: self.trading_pair.asset_1.clone(),
        }
    }

    pub fn delta_2_value(&self) -> ValueVar {
        ValueVar {
            amount: self.delta_2_i.clone(),
            asset_id: self.trading_pair.asset_2.clone(),
        }
    }

    pub fn commit(&self) -> Result<StateCommitmentVar, SynthesisError> {
        // Access constraint system.
        let cs = self.delta_1_i.amount.cs();

        let domain_sep = FqVar::new_constant(cs.clone(), *DOMAIN_SEPARATOR)?;
        let compressed_g_d = self
            .claim_address
            .diversified_generator()
            .compress_to_field()?;

        let inner_hash4 = poseidon377::r1cs::hash_4(
            cs.clone(),
            &domain_sep,
            (
                self.trading_pair.asset_1.asset_id.clone(),
                self.trading_pair.asset_2.asset_id.clone(),
                self.delta_1_i.amount.clone(),
                self.delta_2_i.amount.clone(),
            ),
        )?;

        let inner = poseidon377::r1cs::hash_7(
            cs,
            &domain_sep,
            (
                self.rseed.clone(),
                self.claim_fee.amount.amount.clone(),
                self.claim_fee.asset_id.asset_id.clone(),
                compressed_g_d,
                self.claim_address.transmission_key().compress_to_field()?,
                self.claim_address.clue_key(),
                inner_hash4,
            ),
        )?;

        Ok(StateCommitmentVar { inner })
    }
}

impl AllocVar<SwapPlaintext, Fq> for SwapPlaintextVar {
    fn new_variable<T: std::borrow::Borrow<SwapPlaintext>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let swap_plaintext = f()?.borrow().clone();
        let claim_fee =
            ValueVar::new_variable(cs.clone(), || Ok(swap_plaintext.claim_fee.0), mode)?;
        let delta_1_i = AmountVar::new_variable(cs.clone(), || Ok(swap_plaintext.delta_1_i), mode)?;

        // Note: We currently use `TradingPairVar::new_variable_unchecked` as the only
        // place we use the trading pair is when computing the swap commitment. A malicious
        // prover is unable to switch the direction of the canonical trading pair as the
        // swap commitment integrity check would be invalid.
        let trading_pair = TradingPairVar::new_variable_unchecked(
            cs.clone(),
            || Ok(swap_plaintext.trading_pair),
            mode,
        )?;
        let delta_2_i = AmountVar::new_variable(cs.clone(), || Ok(swap_plaintext.delta_2_i), mode)?;
        let claim_address =
            AddressVar::new_variable(cs.clone(), || Ok(swap_plaintext.claim_address), mode)?;
        let rseed = FqVar::new_variable(
            cs,
            || {
                Ok(Fq::from_le_bytes_mod_order(
                    &swap_plaintext.rseed.to_bytes()[..],
                ))
            },
            mode,
        )?;
        Ok(Self {
            claim_fee,
            delta_1_i,
            trading_pair,
            delta_2_i,
            claim_address,
            rseed,
        })
    }
}

impl DomainType for SwapPlaintext {
    type Proto = pb::SwapPlaintext;
}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        Ok(Self {
            delta_1_i: plaintext
                .delta_1_i
                .ok_or_else(|| anyhow!("missing delta_1_i"))?
                .try_into()?,
            delta_2_i: plaintext
                .delta_2_i
                .ok_or_else(|| anyhow!("missing delta_2_i"))?
                .try_into()?,
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
            rseed: Rseed(plaintext.rseed.as_slice().try_into()?),
        })
    }
}

impl From<SwapPlaintext> for pb::SwapPlaintext {
    fn from(plaintext: SwapPlaintext) -> Self {
        Self {
            delta_1_i: Some(plaintext.delta_1_i.into()),
            delta_2_i: Some(plaintext.delta_2_i.into()),
            claim_fee: Some(plaintext.claim_fee.into()),
            claim_address: Some(plaintext.claim_address.into()),
            trading_pair: Some(plaintext.trading_pair.into()),
            rseed: plaintext.rseed.to_bytes().to_vec(),
        }
    }
}

impl From<&SwapPlaintext> for [u8; SWAP_LEN_BYTES] {
    fn from(swap: &SwapPlaintext) -> [u8; SWAP_LEN_BYTES] {
        let mut bytes = [0u8; SWAP_LEN_BYTES];
        bytes[0..64].copy_from_slice(&swap.trading_pair.to_bytes());
        bytes[64..80].copy_from_slice(&swap.delta_1_i.to_le_bytes());
        bytes[80..96].copy_from_slice(&swap.delta_2_i.to_le_bytes());
        bytes[96..112].copy_from_slice(&swap.claim_fee.0.amount.to_le_bytes());
        bytes[112..144].copy_from_slice(&swap.claim_fee.0.asset_id.to_bytes());
        let pb_address = pb_keys::Address::from(swap.claim_address);
        bytes[144..224].copy_from_slice(&pb_address.inner);
        bytes[224..256].copy_from_slice(&swap.rseed.to_bytes());
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
            anyhow::bail!("incorrect length for serialized swap plaintext");
        }

        let tp_bytes: [u8; 64] = bytes[0..64]
            .try_into()
            .map_err(|_| anyhow!("error fetching trading pair bytes"))?;
        let delta_1_bytes: [u8; 16] = bytes[64..80]
            .try_into()
            .map_err(|_| anyhow!("error fetching delta1 bytes"))?;
        let delta_2_bytes: [u8; 16] = bytes[80..96]
            .try_into()
            .map_err(|_| anyhow!("error fetching delta2 bytes"))?;
        let fee_amount_bytes: [u8; 16] = bytes[96..112]
            .try_into()
            .map_err(|_| anyhow!("error fetching fee amount bytes"))?;
        let fee_asset_id_bytes: [u8; 32] = bytes[112..144]
            .try_into()
            .map_err(|_| anyhow!("error fetching fee asset ID bytes"))?;
        let address_bytes: [u8; 80] = bytes[144..224]
            .try_into()
            .map_err(|_| anyhow!("error fetching address bytes"))?;
        let pb_address = pb_keys::Address {
            inner: address_bytes.to_vec(),
            alt_bech32m: String::new(),
        };
        let rseed: [u8; 32] = bytes[224..256]
            .try_into()
            .map_err(|_| anyhow!("error fetching rseed bytes"))?;

        Ok(SwapPlaintext {
            trading_pair: tp_bytes
                .try_into()
                .map_err(|_| anyhow!("error deserializing trading pair"))?,
            delta_1_i: Amount::from_le_bytes(delta_1_bytes),
            delta_2_i: Amount::from_le_bytes(delta_2_bytes),
            claim_fee: Fee(Value {
                amount: Amount::from_le_bytes(fee_amount_bytes),
                asset_id: asset::Id::try_from(fee_asset_id_bytes)?,
            }),
            claim_address: pb_address.try_into()?,
            rseed: Rseed(rseed),
        })
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
    use penumbra_asset::{asset, Value};
    use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};

    #[test]
    /// Check the swap plaintext can be encrypted and decrypted with the OVK.
    fn swap_encryption_and_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let ovk = fvk.outgoing();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());
        let trading_pair = TradingPair::new(
            asset::Cache::with_known_assets()
                .get_unit("upenumbra")
                .unwrap()
                .id(),
            asset::Cache::with_known_assets()
                .get_unit("nala")
                .unwrap()
                .id(),
        );

        let swap = SwapPlaintext::new(
            &mut rng,
            trading_pair,
            100000u64.into(),
            1u64.into(),
            Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .unwrap()
                    .id(),
            }),
            dest,
        );

        let ciphertext = swap.encrypt(ovk).encrypted_swap;
        let plaintext = SwapCiphertext::decrypt(&ciphertext, ovk, swap.swap_commitment())
            .expect("can decrypt swap");

        assert_eq!(plaintext, swap);
    }
}
