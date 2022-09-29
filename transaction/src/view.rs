pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use decaf377_fmd::Clue;
use penumbra_crypto::{memo::MemoPlaintext, transaction::Fee};
pub use transaction_perspective::TransactionPerspective;

pub struct TransactionView {
    pub actions: Vec<ActionView>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub fmd_clues: Vec<Clue>,
    pub memo: Option<String>,
}
