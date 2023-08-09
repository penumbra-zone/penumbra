use indexed_db_futures::{IdbDatabase, IdbQuerySource};
use indexed_db_futures::prelude::OpenDbRequest;
use rand_core::OsRng;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use penumbra_dex::TradingPair;
use penumbra_proto::client::v1alpha1::simulate_trade_request::routing::Setting::Default;
use penumbra_proto::core::chain::v1alpha1::{ChainParameters, FmdParameters};
use penumbra_proto::core::crypto::v1alpha1::{Address, DenomMetadata, Fee, Value};
use penumbra_proto::core::dex::v1alpha1::SwapClaimPlan;
use penumbra_proto::core::transaction::v1alpha1::{MemoPlaintext, TransactionPlan};
use penumbra_proto::DomainType;
use penumbra_proto::view::v1alpha1::NotesRequest;
use penumbra_shielded_pool::OutputPlan;
use crate::note_record::SpendableNoteRecord;
use crate::planner::Planner;

#[wasm_bindgen]
pub struct WasmPlanner {
    planner:  Planner<OsRng>,
}

#[wasm_bindgen]
impl WasmPlanner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPlanner {
        WasmPlanner {
            planner: Planner::new(OsRng),
        }
    }


    pub fn expiry_height(&mut self, expiry_height: JsValue) -> Result<(), JsValue> {
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

    pub fn swap_claim(&mut self, plan: JsValue) -> Result<(), JsValue> {
        let swap_claim_proto: SwapClaimPlan = serde_wasm_bindgen::from_value(plan)?;

        self.planner.swap_claim(swap_claim_proto.try_into().unwrap());
        Ok(())
    }


    pub fn swap(
        &mut self,
        input_value: JsValue,
        into_denom: JsValue,
        swap_claim_fee: JsValue,
        claim_address: JsValue,
    ) -> Result<(), JsValue> {
        let input_value_proto: Value = serde_wasm_bindgen::from_value(input_value)?;
        let into_denom_proto: DenomMetadata = serde_wasm_bindgen::from_value(into_denom)?;
        let swap_claim_fee_proto: Fee = serde_wasm_bindgen::from_value(swap_claim_fee)?;
        let claim_address_proto: Address = serde_wasm_bindgen::from_value(claim_address)?;


        self.planner.swap(input_value_proto.try_into().unwrap(),
                          into_denom_proto.try_into().unwrap(),
                          swap_claim_fee_proto.try_into().unwrap(),
                          claim_address_proto.try_into().unwrap());

        Ok(())
    }

    pub async fn plan(
        &mut self,
        self_address: JsValue
    ) -> Result<JsValue, JsValue> {

        let self_address_proto: Address = serde_wasm_bindgen::from_value(self_address)?;


        let chain_params_proto: ChainParameters = get_chain_parameters().await.unwrap();
        let fmd_params_proto: FmdParameters =  get_fmd_parameters().await.unwrap();


        let mut spendable_notes = Vec::new();

        let (spendable_requests, voting_requests) =  self.planner.notes_requests();


        for request in spendable_requests {
            let notes = get_notes(request);
            spendable_notes.extend(notes.await.unwrap());
        }

        // Plan the transaction using the gathered information

        let plan: penumbra_transaction::plan::TransactionPlan = self.planner.plan_with_spendable_and_votable_notes(
            &chain_params_proto.try_into().unwrap(),
            &fmd_params_proto.try_into().unwrap(),
            spendable_notes,
            Vec::new(),
            self_address_proto.try_into().unwrap(),
        ).unwrap();

        let plan_proto : TransactionPlan = plan.to_proto();

        Ok(serde_wasm_bindgen::to_value(&plan_proto)?)
    }


}

pub async fn get_chain_parameters() -> Option<ChainParameters> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12).ok()?;

    let db: IdbDatabase = db_req.into_future().await.ok()?;

    let tx = db.transaction_on_one("chain_parameters").ok()?;
    let store = tx.object_store("chain_parameters").ok()?;

    let value: Option<JsValue> = store
        .get_owned("chain_parameters")
        .ok()?
        .await
        .ok()?;

    serde_wasm_bindgen::from_value(value?).ok()?
}

pub async fn get_fmd_parameters() -> Option<FmdParameters> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12).ok()?;

    let db: IdbDatabase = db_req.into_future().await.ok()?;

    let tx = db.transaction_on_one("fmd_parameters").ok()?;
    let store = tx.object_store("fmd_parameters").ok()?;

    let value: Option<JsValue> = store
        .get_owned("fmd")
        .ok()?
        .await
        .ok()?;

    serde_wasm_bindgen::from_value(value?).ok()?
}

pub async fn get_notes(requset: NotesRequest ) -> Option<Vec<SpendableNoteRecord>> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("penumbra", 12).ok()?;

    let db: IdbDatabase = db_req.into_future().await.ok()?;

    let tx = db.transaction_on_one("spendable_notes").ok()?;
    let store = tx.object_store("spendable_notes").ok()?;

    let asset_id = requset.asset_id.unwrap();
    let include_spent = requset.include_spent;

    let values = store.get_all().ok()?.await.ok()?;

    let notes: Vec<SpendableNoteRecord> = values.into_iter()
        .map(|js_value| serde_wasm_bindgen::from_value(js_value).ok())
        .filter_map(|note_option| {
            note_option.and_then(|note: SpendableNoteRecord|
                if note.note.asset_id() == asset_id.clone().try_into().unwrap() && note.height_spent == None {
                    Some(note)
                } else {
                    None
                }
            )
        }).collect();

    Some(notes)

}