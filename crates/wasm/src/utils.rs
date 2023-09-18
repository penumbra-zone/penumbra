use crate::error::WasmResult;
use penumbra_proto::DomainType;
use penumbra_tct::Root;
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// decode SCT root
/// Arguments:
///     tx_bytes: `HEX string`
/// Returns: `penumbra_tct::Root`
#[wasm_bindgen]
pub fn decode_nct_root(tx_bytes: &str) -> Result<JsValue, Error> {
    let root = decode_nct_root_inner(tx_bytes)?;
    serde_wasm_bindgen::to_value(&root)
}

pub fn decode_nct_root_inner(tx_bytes: &str) -> WasmResult<Root> {
    let tx_vec: Vec<u8> = hex::decode(tx_bytes)?;
    let root = penumbra_tct::Root::decode(tx_vec.as_slice())?;
    Ok(root)
}
