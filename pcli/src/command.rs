use structopt::StructOpt;

mod addr;
mod balance;
mod chain;
mod stake;
mod temp;
mod tx;
mod validator;
mod wallet;

pub use addr::AddrCmd;
pub use balance::BalanceCmd;
pub use chain::ChainCmd;
pub use stake::StakeCmd;
pub use temp::TmpCmd;
pub use tx::TxCmd;
pub use validator::ValidatorCmd;
pub use wallet::WalletCmd;

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Creates a transaction.
    Tx(TxCmd),
    /// Manages the wallet state.
    Wallet(WalletCmd),
    /// Manages addresses.
    Addr(AddrCmd),
    /// Synchronizes the client, privately scanning the chain state.
    ///
    /// `pcli` syncs automatically prior to any action requiring chain state,
    /// but this command can be used to "pre-sync" before interactive use.
    Sync,
    /// Displays the current wallet balance.
    Balance(BalanceCmd),
    /// Manages a validator.
    Validator(ValidatorCmd),
    /// Manages delegations and undelegations.
    Stake(StakeCmd),
    /// View chain data.
    Chain(ChainCmd),
    /// Temporary commands for migrating address formats.
    Tmp(TmpCmd),
}

impl Command {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            Command::Tx(cmd) => cmd.needs_sync(),
            Command::Wallet(cmd) => cmd.needs_sync(),
            Command::Addr(cmd) => cmd.needs_sync(),
            Command::Sync => true,
            Command::Balance(cmd) => cmd.needs_sync(),
            Command::Validator(cmd) => cmd.needs_sync(),
            Command::Stake(cmd) => cmd.needs_sync(),
            Command::Tmp(cmd) => cmd.needs_sync(),
            Command::Chain(cmd) => cmd.needs_sync(),
        }
    }
}
