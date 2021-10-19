pub mod constants;
pub mod error;
pub mod output;
pub mod spend;

/// Supported actions in a Penumbra transaction.
pub enum Action {
    Output(output::Output),
    Spend(spend::Spend),
}
