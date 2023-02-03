mod keys;
mod query;
mod tx;
mod validator;
mod view;

pub use keys::KeysCmd;
pub use query::QueryCmd;
pub use tx::TxCmd;
pub use validator::ValidatorCmd;
pub use view::transaction_hashes::TransactionHashesCmd;
pub use view::ViewCmd;

use self::query::OutputFormat;
use crate::App;

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
    #[clap(subcommand)]
    Dev(DevCmd),
}

impl Command {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        match self {
            Command::Transaction(cmd) => cmd.offline(),
            Command::View(cmd) => cmd.offline(),
            Command::Keys(cmd) => cmd.offline(),
            Command::Validator(cmd) => cmd.offline(),
            Command::Query(_) => false,
            Command::Dev(_) => false,
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum DevCmd {
    /// Broadcast a transaction to the network.
    Broadcast {
        #[clap(short, default_value_t, value_enum)]
        format: OutputFormat,
    },
    BroadcastFixed,
}

impl DevCmd {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        use penumbra_proto::DomainType;
        use penumbra_transaction::Transaction;
        use std::io::Read;

        match self {
            DevCmd::Broadcast { format } => {
                let tx = match format {
                    OutputFormat::Json => {
                        let mut input = String::new();
                        std::io::stdin().read_to_string(&mut input)?;
                        serde_json::from_str(&input)?
                    }
                    OutputFormat::Base64 => {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        let bytes = base64::decode(input.trim())?;
                        Transaction::decode(bytes.as_slice())?
                    }
                };

                app.submit_transaction(&tx, None).await?;
            }
            DevCmd::BroadcastFixed => {
                let b64 = "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";
                let bytes = base64::decode(&b64)?;
                let tx = Transaction::decode(bytes.as_slice())?;
                app.submit_transaction(&tx, None).await?;
            }
        }
        Ok(())
    }
}
