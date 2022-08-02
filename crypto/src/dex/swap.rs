mod ciphertext;
mod plaintext;

use anyhow::Result;
use ark_ff::PrimeField;
pub use ciphertext::SwapCiphertext;
use decaf377::{Element, FieldExt, Fq};
use decaf377_ka::Public;
pub use plaintext::SwapPlaintext;

use once_cell::sync::Lazy;
use poseidon377::hash_5;

use crate::asset;
use penumbra_proto::{dex as pb, Protobuf};

use super::TradingPair;

// Swap ciphertext byte length
pub const SWAP_CIPHERTEXT_BYTES: usize = 184;
// Swap plaintext byte length
pub const SWAP_LEN_BYTES: usize = 168;
pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

/// The nonce used for swap encryption.
///
/// The nonce will always be `[9u8; 12]` which is okay since we use a new
/// ephemeral key each time.
pub static SWAP_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| [9u8; 12]);

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));

// Constructs the unique asset ID for a swap as a poseidon hash of the input data for the swap.
//
// https://protocol.penumbra.zone/main/zswap/swap.html#swap-actions
pub fn generate_swap_asset_id(
    delta_1: u64,
    delta_2: u64,
    fee: u64,
    b_d: Element,
    pk_d: Public,
    trading_pair: TradingPair,
) -> Result<asset::Id> {
    let packed_values = {
        let mut bytes = [0u8; 24];
        bytes[0..8].copy_from_slice(&delta_1.to_le_bytes());
        bytes[8..16].copy_from_slice(&delta_2.to_le_bytes());
        bytes[16..24].copy_from_slice(&fee.to_le_bytes());
        Fq::from_le_bytes_mod_order(&bytes)
    };

    let asset_id_hash = hash_5(
        &DOMAIN_SEPARATOR,
        (
            trading_pair.asset_1().0,
            trading_pair.asset_2().0,
            packed_values,
            b_d.compress_to_field(),
            Fq::from_bytes(pk_d.0)?,
        ),
    );

    Ok(asset::Id(asset_id_hash))
}

#[derive(Clone, Debug, Copy)]
pub struct BatchSwapOutputData {
    pub delta_1: u64,
    pub delta_2: u64,
    pub lambda_1: u64,
    pub lambda_2: u64,
    pub success: bool,
}

impl Protobuf<pb::BatchSwapOutputData> for BatchSwapOutputData {}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            lambda_1: s.lambda_1,
            lambda_2: s.lambda_2,
            success: s.success,
        }
    }
}

impl TryFrom<pb::BatchSwapOutputData> for BatchSwapOutputData {
    type Error = anyhow::Error;
    fn try_from(s: pb::BatchSwapOutputData) -> Result<Self, Self::Error> {
        Ok(Self {
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            lambda_1: s.lambda_1,
            lambda_2: s.lambda_2,
            success: s.success,
        })
    }
}
