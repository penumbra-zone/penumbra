use rand_core::OsRng;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use penumbra_proto::core::crypto::v1alpha1::{Address, Fee, Value};
use penumbra_proto::core::transaction::v1alpha1::MemoPlaintext;
use penumbra_shielded_pool::OutputPlan;
use crate::planner::Planner;

#[wasm_bindgen]
pub struct WasmPlanner {
    planner: Planner<OsRng>,
}

#[wasm_bindgen]
impl WasmPlanner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPlanner {
        WasmPlanner {
            planner: Planner::new(OsRng),
        }
    }


    pub fn expiry_height(&mut self, expiry_height: JsValue)-> Result<(), JsValue>{

        self.planner.expiry_height(serde_wasm_bindgen::from_value(expiry_height)?);

        Ok(())

    }


    pub fn memo(&mut self, memo: JsValue) -> Result<(), JsValue> {
        let memo_proto: MemoPlaintext = serde_wasm_bindgen::from_value(memo)?;

        self.planner.memo(memo_proto.try_into().unwrap());

        Ok(())
    }


    pub fn fee(&mut self, fee: JsValue) -> Result<(), JsValue> {
        let fee_proto: Fee = serde_wasm_bindgen::from_value(fee)?;

        self.planner.fee(fee_proto.try_into().unwrap());

        Ok(())

    }

    pub fn output(&mut self, value: JsValue, address: JsValue) -> Result<(), JsValue> {
        let value_proto: Value = serde_wasm_bindgen::from_value(value)?;
        let address_proto: Address = serde_wasm_bindgen::from_value(address)?;

        self.planner.output(value_proto.try_into().unwrap(), address_proto.try_into().unwrap());

        Ok(())
    }
}