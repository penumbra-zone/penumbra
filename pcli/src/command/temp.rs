use anyhow::Result;
use structopt::StructOpt;

use penumbra_crypto::parse_v0_testnet_address;

#[derive(Debug, StructOpt)]
pub enum TmpCmd {
    /// Migrate Penumbra testnet address from v0 to v1 format.
    AddressMigrate {
        /// The v0 address to migrate.
        address: String,
    },
}

impl TmpCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            TmpCmd::AddressMigrate { .. } => false,
        }
    }

    pub async fn exec(&self) -> Result<()> {
        match self {
            TmpCmd::AddressMigrate { address } => match parse_v0_testnet_address(address.clone()) {
                Ok(new_address) => println!("{}", new_address),
                Err(err) => return Err(err),
            },
        }
        Ok(())
    }
}
