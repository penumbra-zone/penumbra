use penumbra_crypto::memo::MemoPlaintext;

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
pub use transaction_perspective::TransactionPerspective;

pub struct TransactionView {
    pub actions: Vec<ActionView>,
}
