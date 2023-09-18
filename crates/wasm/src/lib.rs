#![deny(clippy::unwrap_used)]
#![allow(dead_code)]
extern crate core;

mod note_record;
mod planner;
mod swap_record;
mod tx;
mod utils;
mod view_server;
use penumbra_proto::{core::crypto::v1alpha1 as pb, serializers::bech32str, DomainType};

use penumbra_keys::{Address, FullViewingKey};
use std::convert::TryFrom;
use std::str::FromStr;

use penumbra_keys::keys::{SeedPhrase, SpendKey};
use wasm_bindgen::prelude::*;

use penumbra_transaction::Transaction;

pub use tx::send_plan;
pub use view_server::ViewServer;

#[wasm_bindgen]
pub fn generate_spend_key(seed_phrase: &str) -> JsValue {
    utils::set_panic_hook();
    let seed =
        SeedPhrase::from_str(seed_phrase).expect("the provided string is a valid seed phrase");
    let spend_key = SpendKey::from_seed_phrase_bip39(seed, 0);

    let proto = spend_key.to_proto();
    let spend_key_str = &bech32str::encode(
        &proto.inner,
        bech32str::spend_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    serde_wasm_bindgen::to_value(&spend_key_str).expect("able to serialize spend key")
}

#[wasm_bindgen]
pub fn get_full_viewing_key(spend_key_str: &str) -> JsValue {
    utils::set_panic_hook();
    let spend_key =
        SpendKey::from_str(spend_key_str).expect("the provided string is a valid spend key");

    let fvk: &FullViewingKey = spend_key.full_viewing_key();

    let proto = pb::FullViewingKey::from(fvk.to_proto());

    let fvk_str = &bech32str::encode(
        &proto.inner,
        bech32str::full_viewing_key::BECH32_PREFIX,
        bech32str::Bech32m,
    );
    serde_wasm_bindgen::to_value(&fvk_str).expect("able to serialize full viewing key")
}

#[wasm_bindgen]
pub fn get_address_by_index(full_viewing_key: &str, index: u32) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .expect("the provided string is a valid FullViewingKey");

    let (address, _dtk) = fvk.incoming().payment_address(index.into());

    let proto = address.to_proto();
    let address_str = &bech32str::encode(
        &proto.inner,
        bech32str::address::BECH32_PREFIX,
        bech32str::Bech32m,
    );

    serde_wasm_bindgen::to_value(&address_str).expect("able to serialize address")
}

#[wasm_bindgen]
pub fn base64_to_bech32(prefix: &str, base64_str: &str) -> JsValue {
    utils::set_panic_hook();

    let bech32 = &bech32str::encode(
        &base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str)
            .expect("the provided string is a valid base64 string"),
        prefix,
        bech32str::Bech32m,
    );
    serde_wasm_bindgen::to_value(bech32).expect("able to serialize bech32 string")
}
#[wasm_bindgen]
pub fn is_controlled_address(full_viewing_key: &str, address: &str) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .expect("the provided string is a valid FullViewingKey");

    let index = fvk.address_index(&Address::from_str(address.as_ref()).expect("valid address"));

    serde_wasm_bindgen::to_value(&index).expect("able to serialize address index")
}

#[wasm_bindgen]
pub fn get_short_address_by_index(full_viewing_key: &str, index: u32) -> JsValue {
    utils::set_panic_hook();
    let fvk = FullViewingKey::from_str(full_viewing_key.as_ref())
        .expect("The provided string is not a valid FullViewingKey");

    let (address, _dtk) = fvk.incoming().payment_address(index.into());
    let short_address = address.display_short_form();
    serde_wasm_bindgen::to_value(&short_address).expect("able to serialize address")
}

#[wasm_bindgen]
pub fn decode_transaction(tx_bytes: &str) -> JsValue {
    utils::set_panic_hook();
    let tx_vec: Vec<u8> =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, tx_bytes)
            .expect("the provided tx string is a valid base64 string");
    let transaction: Transaction =
        Transaction::try_from(tx_vec).expect("the provided tx string is a valid transaction");
    serde_wasm_bindgen::to_value(&transaction).expect("able to serialize transaction")
}

#[wasm_bindgen]
pub fn decode_nct_root(tx_bytes: &str) -> JsValue {
    utils::set_panic_hook();
    let tx_vec: Vec<u8> =
        hex::decode(tx_bytes).expect("the provided tx string is a valid hex string");
    let root = penumbra_tct::Root::decode(tx_vec.as_slice())
        .expect("the provided tx string is a valid nct root");
    serde_wasm_bindgen::to_value(&root).expect("able to serialize nct root")
}
