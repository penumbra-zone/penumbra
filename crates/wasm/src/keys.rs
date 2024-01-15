use std::str::FromStr;

use anyhow;
use rand_core::OsRng;
use wasm_bindgen::prelude::*;

use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_keys::{Address, FullViewingKey};
use penumbra_proof_params::{
    CONVERT_PROOF_PROVING_KEY, DELEGATOR_VOTE_PROOF_PROVING_KEY,
    NULLIFIER_DERIVATION_PROOF_PROVING_KEY, OUTPUT_PROOF_PROVING_KEY, SPEND_PROOF_PROVING_KEY,
    SWAPCLAIM_PROOF_PROVING_KEY, SWAP_PROOF_PROVING_KEY,
};
use penumbra_proto::{core::keys::v1alpha1 as pb, serializers::bech32str, DomainType};
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::error::WasmResult;
use crate::utils;

/// Loads the proving key as a collection of bytes, and to sets the keys in memory
/// dynamicaly at runtime. Failure to bundle the proving keys in the wasm binary
/// or call the load function will fail to generate a proof. Consumers of this
/// function will additionally require downloading the proving key parameter `.bin`
/// file for each key type.
#[wasm_bindgen]
pub fn load_proving_key(parameters: JsValue, key_type: &str) -> WasmResult<()> {
    // Deserialize JsValue into Vec<u8>.
    let parameters_bytes: Vec<u8> = Uint8Array::new(&parameters).to_vec();

    // Map key type with proving keys.
    let proving_key_map = match key_type {
        "spend" => &SPEND_PROOF_PROVING_KEY,
        "output" => &OUTPUT_PROOF_PROVING_KEY,
        "delegator_vote" => &DELEGATOR_VOTE_PROOF_PROVING_KEY,
        "nullifier_derivation" => &NULLIFIER_DERIVATION_PROOF_PROVING_KEY,
        "swap" => &SWAP_PROOF_PROVING_KEY,
        "swap_claim" => &SWAPCLAIM_PROOF_PROVING_KEY,
        "convert" => &CONVERT_PROOF_PROVING_KEY,
        _ => return Err(anyhow::anyhow!("Unsupported key type").into()),
    };

    // Load proving key.
    proving_key_map.try_load_unchecked(&parameters_bytes)?;
    Ok(())
}

/// generate a spend key from a seed phrase
/// Arguments:
///     seed_phrase: `string`
/// Returns: `bech32 string`
#[wasm_bindgen]
pub fn generate_spend_key(seed_phrase: &str) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let seed = SeedPhrase::from_str(seed_phrase)?;
    let path = Bip44Path::new(0);
    let spend_key = SpendKey::from_seed_phrase_bip44(seed, &path);

    let proto = spend_key.to_proto();

    let spend_key_str = bech32str::encode(
        &proto.inner,
        bech32str::spend_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    Ok(JsValue::from_str(&spend_key_str))
}

/// get full viewing key from spend key
/// Arguments:
///     spend_key_str: `bech32 string`
/// Returns: `bech32 string`
#[wasm_bindgen]
pub fn get_full_viewing_key(spend_key: &str) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let spend_key = SpendKey::from_str(spend_key)?;

    let fvk: &FullViewingKey = spend_key.full_viewing_key();

    let proto = fvk.to_proto();

    let fvk_bech32 = bech32str::encode(
        &proto.inner,
        bech32str::full_viewing_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );
    Ok(JsValue::from_str(&fvk_bech32))
}

/// Wallet id: the hash of a full viewing key, used as an account identifier
/// Arguments:
///     full_viewing_key: `bech32 string`
/// Returns: `bech32 string`
#[wasm_bindgen]
pub fn get_wallet_id(full_viewing_key: &str) -> WasmResult<String> {
    utils::set_panic_hook();

    let fvk = FullViewingKey::from_str(full_viewing_key)?;
    Ok(fvk.wallet_id().to_string())
}

/// get address by index using FVK
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `pb::Address`
#[wasm_bindgen]
pub fn get_address_by_index(full_viewing_key: &str, index: u32) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let fvk = FullViewingKey::from_str(full_viewing_key)?;
    let (address, _dtk) = fvk.incoming().payment_address(index.into());
    let proto = address.to_proto();
    let result = serde_wasm_bindgen::to_value(&proto)?;
    Ok(result)
}

/// get ephemeral (randomizer) address using FVK
/// The derivation tree is like "spend key / address index / ephemeral address" so we must also pass index as an argument
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `pb::Address`
#[wasm_bindgen]
pub fn get_ephemeral_address(full_viewing_key: &str, index: u32) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let fvk = FullViewingKey::from_str(full_viewing_key)?;
    let (address, _dtk) = fvk.ephemeral_address(OsRng, index.into());
    let proto = address.to_proto();
    let result = serde_wasm_bindgen::to_value(&proto)?;
    Ok(result)
}

/// Check if the address is FVK controlled
/// Arguments:
///     full_viewing_key: `bech32 String`
///     address: `bech32 String`
/// Returns: `Option<pb::AddressIndex>`
#[wasm_bindgen]
pub fn is_controlled_address(full_viewing_key: &str, address: &str) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let fvk = FullViewingKey::from_str(full_viewing_key)?;
    let index: Option<pb::AddressIndex> = fvk
        .address_index(&Address::from_str(address)?)
        .map(Into::into);
    let result = serde_wasm_bindgen::to_value(&index)?;
    Ok(result)
}

/// Get canonical short form address by index
/// This feature is probably redundant and will be removed from wasm in the future
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `String`
#[wasm_bindgen]
pub fn get_short_address_by_index(full_viewing_key: &str, index: u32) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let fvk = FullViewingKey::from_str(full_viewing_key)?;
    let (address, _dtk) = fvk.incoming().payment_address(index.into());
    let short_address = address.display_short_form();
    Ok(JsValue::from_str(&short_address))
}
