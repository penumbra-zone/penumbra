use serde_wasm_bindgen::Error;
use thiserror::Error;
use web_sys::DomException;

#[derive(Debug)]
pub struct DomError(DomException);

impl std::fmt::Display for DomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DOM Exception: {:?}", self.0)
    }
}

impl std::error::Error for DomError {}

impl From<DomException> for WasmError {
    fn from(dom_exception: DomException) -> Self {
        WasmError::Dom(DomError(dom_exception))
    }
}
// --------------- //

pub type WasmResult<T> = Result<T, WasmError>;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    Dom(#[from] DomError),
}

impl From<WasmError> for serde_wasm_bindgen::Error {
    fn from(wasm_err: WasmError) -> Self {
        Error::new(wasm_err.to_string())
    }
}
