use crate::error::WasmResult;
use penumbra_keys::keys::{SeedPhrase, SpendKey};
use penumbra_keys::{Address, FullViewingKey};
use penumbra_proto::{core::crypto::v1alpha1 as pb, serializers::bech32str, DomainType};
use rand_core::OsRng;
use serde_wasm_bindgen::Error;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// generate a spend key from a seed phrase
/// Arguments:
///     seed_phrase: `string`
/// Returns: `bech32 string`
#[wasm_bindgen(js_name = generateSpendKey)]
pub fn generate_spend_key(seed_phrase: &str) -> Result<JsValue, Error> {
    let result = spend_key_from_seed(seed_phrase)?;

    Ok(JsValue::from_str(&result))
}

/// get full viewing key from spend key
/// Arguments:
///     spend_key_str: `bech32 string`
/// Returns: `bech32 string`
#[wasm_bindgen(js_name = getFullViewingKey)]
pub fn get_full_viewing_key(spend_key: &str) -> Result<JsValue, Error> {
    let result = get_full_viewing_key_from_spend(spend_key)?;
    Ok(JsValue::from_str(&result))
}

/// get address by index using FVK
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `pb::Address`
#[wasm_bindgen(js_name = getAddressByIndex)]
pub fn get_address_by_index(full_viewing_key: &str, index: u32) -> Result<JsValue, Error> {
    let result = address_by_index(full_viewing_key, index)?;
    serde_wasm_bindgen::to_value(&result)
}

/// get ephemeral (randomizer) address using FVK
/// The derivation tree is like "spend key / address index / ephemeral address" so we must also pass index as an argument
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `pb::Address`
#[wasm_bindgen(js_name = getEphemeralAddress)]
pub fn get_ephemeral_address(full_viewing_key: &str, index: u32) -> Result<JsValue, Error> {
    let result = ephemeral_address(full_viewing_key, index)?;
    serde_wasm_bindgen::to_value(&result)
}

/// Check if the address is FVK controlled
/// Arguments:
///     full_viewing_key: `bech32 String`
///     address: `bech32 String`
/// Returns: `Option<pb::AddressIndex>`
#[wasm_bindgen(js_name = isControlledAddress)]
pub fn is_controlled_address(full_viewing_key: &str, address: &str) -> Result<JsValue, Error> {
    let result = address_index(full_viewing_key, address)?;
    serde_wasm_bindgen::to_value(&result)
}

/// Get canonical short form address by index
/// This feature is probably redundant and will be removed from wasm in the future
/// Arguments:
///     full_viewing_key: `bech32 string`
///     index: `u32`
/// Returns: `String`
#[wasm_bindgen(js_name = getShortAddressByIndex)]
pub fn get_short_address_by_index(full_viewing_key: &str, index: u32) -> Result<JsValue, Error> {
    let result = short_address_by_index(full_viewing_key, index)?;
    Ok(JsValue::from_str(&result))
}

pub fn spend_key_from_seed(seed_phrase: &str) -> WasmResult<String> {
    let seed = SeedPhrase::from_str(seed_phrase)?;
    let spend_key = SpendKey::from_seed_phrase_bip39(seed, 0);

    let proto = spend_key.to_proto();

    let spend_key_str = bech32str::encode(
        &proto.inner,
        bech32str::spend_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    Ok(spend_key_str)
}

pub fn get_full_viewing_key_from_spend(spend_key_bech32: &str) -> WasmResult<String> {
    let spend_key = SpendKey::from_str(spend_key_bech32)?;

    let fvk: &FullViewingKey = spend_key.full_viewing_key();

    let proto = fvk.to_proto();

    let fvk_bech32 = bech32str::encode(
        &proto.inner,
        bech32str::full_viewing_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );
    Ok(fvk_bech32)
}

pub fn address_by_index(full_viewing_key: &str, index: u32) -> WasmResult<pb::Address> {
    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    let (address, _dtk) = fvk.incoming().payment_address(index.into());

    let proto = address.to_proto();

    Ok(proto)
}

pub fn ephemeral_address(full_viewing_key: &str, index: u32) -> WasmResult<pb::Address> {
    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    let (address, _dtk) = fvk.ephemeral_address(OsRng, index.into());

    let proto = address.to_proto();

    Ok(proto)
}

pub fn address_index(
    full_viewing_key: &str,
    address: &str,
) -> WasmResult<Option<pb::AddressIndex>> {
    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    let index = fvk
        .address_index(&Address::from_str(address)?)
        .map(Into::into);
    Ok(index)
}

pub fn short_address_by_index(full_viewing_key: &str, index: u32) -> WasmResult<String> {
    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    let (address, _dtk) = fvk.incoming().payment_address(index.into());
    let short_address = address.display_short_form();
    Ok(short_address)
}
