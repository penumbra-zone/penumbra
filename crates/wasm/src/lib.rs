#![allow(dead_code)]
extern crate core;

mod note_record;
mod planner;
mod swap_record;
mod tx;
mod utils;
mod view_server;
use penumbra_proto::{core::crypto::v1alpha1 as pb, serializers::bech32str, DomainType};

use penumbra_crypto::{Address, FullViewingKey};
use std::convert::TryFrom;
use std::str::FromStr;

use anyhow::Context;
use penumbra_crypto::keys::{SeedPhrase, SpendKey};
use wasm_bindgen::prelude::*;

use penumbra_transaction::Transaction;

pub use tx::send_plan;
pub use view_server::ViewServer;

#[wasm_bindgen]
pub fn generate_spend_key(seed_phrase: &str) -> JsValue {
    utils::set_panic_hook();
    let seed = SeedPhrase::from_str(seed_phrase).unwrap();
    let spend_key = SpendKey::from_seed_phrase(seed, 0);

    let proto = spend_key.to_proto();
    let spend_key_str = &bech32str::encode(
        &proto.inner,
        bech32str::spend_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    return serde_wasm_bindgen::to_value(&spend_key_str).unwrap();
}

#[wasm_bindgen]
pub fn get_full_viewing_key(spend_key_str: &str) -> JsValue {
    utils::set_panic_hook();
    let spend_key = SpendKey::from_str(spend_key_str).unwrap();

    let fvk: &FullViewingKey = spend_key.full_viewing_key();

    let proto = pb::FullViewingKey::from(fvk.to_proto());

    let fvk_str = &bech32str::encode(
        &proto.inner,
        bech32str::full_viewing_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );
    return serde_wasm_bindgen::to_value(&fvk_str).unwrap();
}

#[wasm_bindgen]
pub fn get_address_by_index(full_viewing_key: &str, index: u32) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .context("The provided string is not a valid FullViewingKey")
        .unwrap();

    let (address, _dtk) = fvk.incoming().payment_address(index.into());

    let proto = address.to_proto();
    let address_str = &bech32str::encode(
        &proto.inner,
        bech32str::address::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    return serde_wasm_bindgen::to_value(&address_str).unwrap();
}

#[wasm_bindgen]
pub fn base64_to_bech32(prefix: &str, base64_str: &str) -> JsValue {
    utils::set_panic_hook();

    let bech32 = &bech32str::encode(
        &base64::decode(base64_str).unwrap(),
        prefix,
        bech32str::Bech32m,
    );
    return serde_wasm_bindgen::to_value(bech32).unwrap();
}
#[wasm_bindgen]
pub fn is_controlled_address(full_viewing_key: &str, address: &str) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .context("The provided string is not a valid FullViewingKey")
        .unwrap();

    let index = fvk.address_index(&Address::from_str(address.as_ref()).unwrap());

    return serde_wasm_bindgen::to_value(&index).unwrap();
}

#[wasm_bindgen]
pub fn get_short_address_by_index(full_viewing_key: &str, index: u32) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .context("The provided string is not a valid FullViewingKey")
        .unwrap();

    let (address, _dtk) = fvk.incoming().payment_address(index.into());
    let short_address = address.display_short_form();
    return serde_wasm_bindgen::to_value(&short_address).unwrap();
}

#[wasm_bindgen]
pub fn decode_transaction(tx_bytes: &str) -> JsValue {
    utils::set_panic_hook();
    let tx_vec: Vec<u8> = base64::decode(tx_bytes).unwrap();
    let transaction: Transaction = Transaction::try_from(tx_vec).unwrap();
    return serde_wasm_bindgen::to_value(&transaction).unwrap();
}

#[wasm_bindgen]
pub fn decode_nct_root(tx_bytes: &str) -> JsValue {
    utils::set_panic_hook();
    let tx_vec: Vec<u8> = hex::decode(tx_bytes).unwrap();
    let root = penumbra_tct::Root::decode(tx_vec.as_slice()).unwrap();
    return serde_wasm_bindgen::to_value(&root).unwrap();
}
