use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_proto::DomainType;

use penumbra_extension::error::WasmResult;
use penumbra_extension::utils;

/// decode SCT root
/// Arguments:
///     tx_bytes: `HEX string`
/// Returns: `penumbra_tct::Root`
#[wasm_bindgen]
pub fn decode_sct_root(tx_bytes: &str) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let tx_vec: Vec<u8> = hex::decode(tx_bytes)?;
    let root = penumbra_tct::Root::decode(tx_vec.as_slice())?;
    let result = serde_wasm_bindgen::to_value(&root)?;
    Ok(result)
}
