pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use decaf377_fmd::Clue;
use penumbra_crypto::transaction::Fee;
pub use transaction_perspective::TransactionPerspective;

use crate::Transaction;

pub struct TransactionView {
    pub tx: Transaction,
    pub actions: Vec<ActionView>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub fmd_clues: Vec<Clue>,
    pub memo: Option<String>,
}
