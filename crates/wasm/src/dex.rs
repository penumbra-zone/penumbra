use crate::utils;
use penumbra_dex::lp::position::Position;
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// compute position id
/// Arguments:
///     position: `Position`
/// Returns: `PositionId`
#[wasm_bindgen]
pub fn compute_position_id(position: JsValue) -> Result<JsValue, Error> {
    utils::set_panic_hook();

    let position: Position = serde_wasm_bindgen::from_value(position)?;
    serde_wasm_bindgen::to_value(&position.id())
}
