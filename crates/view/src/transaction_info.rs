use penumbra_sdk_transaction::{
    txhash::TransactionId, Transaction, TransactionPerspective, TransactionSummary, TransactionView,
};

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    // The height the transaction was included in a block, if known.
    pub height: u64,
    // The hash of the transaction.
    pub id: TransactionId,
    // The transaction data itself.
    pub transaction: Transaction,
    // The transaction perspective, as seen by this view server.
    pub perspective: TransactionPerspective,
    // A precomputed transaction view of `transaction` from `perspective`, included for convenience of clients that don't have support for viewing transactions on their own.
    pub view: TransactionView,
    // A short view of balances leaving and entering accounts, computed from transaction actions.
    pub summary: TransactionSummary,
}
