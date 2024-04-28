pub use debug::DebugCmd;
pub use init::InitCmd;
pub use query::QueryCmd;
pub use threshold::ThresholdCmd;
pub use tx::TxCmd;
pub use validator::ValidatorCmd;
pub use view::ViewCmd;

use self::ceremony::CeremonyCmd;

mod ceremony;
mod debug;
mod init;
mod query;
mod threshold;
mod tx;
mod utils;
mod validator;
mod view;

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
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Initialize `pcli` with a new wallet, or reset it.
    ///
    /// This command requires selecting a custody backend.  The `SoftKMS`
    /// backend is a good default choice.  More backends (e.g., threshold
    /// custody, hardware wallets) may be added in the future.
    #[clap(display_order = 100)]
    Init(InitCmd),
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
    /// Manage a validator.
    #[clap(subcommand, display_order = 900)]
    Validator(ValidatorCmd),
    /// Display information related to diagnosing problems running Penumbra
    #[clap(subcommand, display_order = 999)]
    Debug(DebugCmd),
    /// Contribute to the summoning ceremony.
    #[clap(subcommand, display_order = 990)]
    Ceremony(CeremonyCmd),
    /// Follow the threshold signing protocol.
    #[clap(subcommand, display_order = 500)]
    Threshold(ThresholdCmd),
}

impl Command {
    /// Determine if this command can run in "offline" mode.
    pub fn offline(&self) -> bool {
        match self {
            Command::Init(_) => true,
            Command::Transaction(cmd) => cmd.offline(),
            Command::View(cmd) => cmd.offline(),
            Command::Validator(cmd) => cmd.offline(),
            Command::Query(cmd) => cmd.offline(),
            Command::Debug(cmd) => cmd.offline(),
            Command::Ceremony(_) => false,
            Command::Threshold(cmd) => cmd.offline(),
        }
    }
}
