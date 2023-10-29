use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use penumbra_proto::DomainType;

use crate::error::WasmResult;
use crate::utils;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

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
