use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum WalletCmd {
    /// Import an existing spend seed.
    Import {
        /// A 32-byte hex string encoding the spend seed.
        spend_seed: String,
    },
    /// Export the spend seed for the wallet.
    Export,
    /// Generate a new spend seed.
    Generate,
    /// Keep the spend seed, but reset all other client state.
    Reset,
    /// Delete the entire wallet permanently.
    Delete,
}

impl WalletCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            WalletCmd::Import { .. } => false,
            WalletCmd::Export => false,
            WalletCmd::Generate => false,
            WalletCmd::Reset => false,
            WalletCmd::Delete => false,
        }
    }
}
