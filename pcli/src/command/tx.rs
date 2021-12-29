use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// The destination address to send funds to.
        #[structopt(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        from: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[structopt(long)]
        memo: Option<String>,
    },
}

impl TxCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            TxCmd::Send { .. } => true,
        }
    }
}
