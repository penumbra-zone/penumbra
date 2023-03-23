mod debug;
mod keys;
mod query;
mod tx;
mod validator;
mod view;

pub use debug::DebugCmd;
pub use keys::KeysCmd;
pub use query::QueryCmd;
pub use tx::TxCmd;
pub use validator::ValidatorCmd;
pub use view::transaction_hashes::TransactionHashesCmd;
pub use view::ViewCmd;

// Note on display_order:
//
// The value is between 0 and 999 (the default).  Sorting of subcommands is done
// by display_order first, and then alphabetically.  We should not try to order
// every set of subcommands -- for instance, it doesn't make sense to try to
// impose a non-alphabetical ordering on the query subcommands -- but we can use
// the order to group related commands.
//
// Setting spaced numbers is future-proofing, letting us insert other commands
// without noisy renumberings.
//
// https://docs.rs/clap/latest/clap/builder/struct.App.html#method.display_order

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Query the public chain state, like the validator set.
    ///
    /// This command has two modes: it can be used to query raw bytes of
    /// arbitrary keys with the `key` subcommand, or it can be used to query
    /// typed data with a subcommand for a particular component.
    #[clap(subcommand, display_order = 200, visible_alias = "q")]
    Query(QueryCmd),
    /// View your private chain state, like account balances.
    #[clap(subcommand, display_order = 300, visible_alias = "v")]
    View(ViewCmd),
    /// Create and broadcast a transaction.
    #[clap(subcommand, display_order = 400, visible_alias = "tx")]
    Transaction(TxCmd),
    /// Manage your wallet's keys.
    #[clap(subcommand, display_order = 500)]
    Keys(KeysCmd),
    /// Manage a validator.
    #[clap(subcommand, display_order = 998)]
    Validator(ValidatorCmd),
    /// Display information related to diagnosing problems running Penumbra
    #[clap(subcommand, display_order = 999)]
    Debug(DebugCmd),
}

impl Command {
    /// Determine if this command can run in "offline" mode.
    pub fn offline(&self) -> bool {
        match self {
            Command::Transaction(cmd) => cmd.offline(),
            Command::View(cmd) => cmd.offline(),
            Command::Keys(cmd) => cmd.offline(),
            Command::Validator(cmd) => cmd.offline(),
            Command::Query(_) => false,
            Command::Debug(cmd) => cmd.offline(),
        }
    }
}
