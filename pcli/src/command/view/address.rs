use anyhow::Result;
use rand_core::OsRng;

use penumbra_crypto::FullViewingKey;

#[derive(Debug, clap::Parser)]
pub struct AddressCmd {
    /// Show the address with a particular numerical index [default: 0].
    index: u32,
    /// Generate an ephemeral address instead of an indexed one.
    #[clap(short, long)]
    ephemeral: bool,
}

impl AddressCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        match self.ephemeral {
            false => {
                let (address, _dtk) = fvk.incoming().payment_address(self.index.into());
                println!("{address}");
            }
            true => {
                let (address, _dtk) = fvk.incoming().ephemeral_address(OsRng, self.index.into());
                println!("{address}");
            }
        }

        Ok(())
    }
}
