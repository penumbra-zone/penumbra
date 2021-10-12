use crate::Output;
use crate::Spend;

/// Used to construct a Penumbra transaction.
pub struct TransactionBuilder {
    spends: Vec<Spend>,
    // Order of output descriptions should be randomized.
    outputs: Vec<Output>,
    // Or vec?
    fee: u32,
    // Put chain_id and anchor here too?
}

impl TransactionBuilder {
    pub fn add_spend() {
        todo!()
    }

    pub fn add_output() {
        todo!()
    }

    pub fn set_fee() {
        todo!()
    }
}
