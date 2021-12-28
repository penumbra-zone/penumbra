#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Chain ID not set")]
    NoChainID,
    #[error("Fee not set")]
    FeeNotSet,
    #[error("Value balance of this transaction is not zero")]
    NonZeroValueBalance,
}
