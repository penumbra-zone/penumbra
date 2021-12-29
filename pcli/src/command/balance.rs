use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct BalanceCmd {
    /// If set, breaks down balances by address.
    #[structopt(short, long)]
    pub by_address: bool,
    #[structopt(long)]
    /// If set, does not attempt to synchronize the wallet before printing the balance.
    pub offline: bool,
}

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        self.offline
    }
}
