mod keys;
mod query;
mod tx;
mod validator;
mod view;

pub use keys::KeysCmd;
pub use query::QueryCmd;
pub use tx::TxCmd;
pub use validator::ValidatorCmd;
pub use view::ViewCmd;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Create and broadcast a transaction.
    #[clap(subcommand)]
    Tx(TxCmd),
    /// View your private state.
    #[clap(subcommand)]
    View(ViewCmd),
    /// Manage your wallet's keys.
    #[clap(subcommand)]
    Keys(KeysCmd),
    /// Query the public chain state.
    ///
    /// This command has two modes: it can be used to query raw bytes of
    /// arbitrary keys with the `key` subcommand, or it can be used to query
    /// typed data with a subcommand for a particular component.
    #[clap(subcommand)]
    Query(QueryCmd),
    ///
    /// Synchronizes the client, privately scanning the chain state.
    ///
    /// `pcli` syncs automatically prior to any action requiring chain state,
    /// but this command can be used to "pre-sync" before interactive use.
    Sync,
    /// Manage a validator.
    #[clap(subcommand)]
    Validator(ValidatorCmd),
}

impl Command {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            Command::Tx(cmd) => cmd.needs_sync(),
            Command::View(cmd) => cmd.needs_sync(),
            Command::Keys(cmd) => cmd.needs_sync(),
            Command::Sync => true,
            Command::Validator(cmd) => cmd.needs_sync(),
            Command::Query(_) => false,
        }
    }
}
