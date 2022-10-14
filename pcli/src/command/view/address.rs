use anyhow::Result;
use rand_core::OsRng;

use penumbra_crypto::FullViewingKey;

#[derive(Debug, clap::Parser)]
pub struct AddressCmd {
    /// Show the address with a particular numerical index [default: 0].
    index: Option<u64>,
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
                let (address, _dtk) = fvk
                    .incoming()
                    .payment_address(self.index.unwrap_or(0).into());
                println!("{}", address);
            }
            true => {
                if self.index.is_some() {
                    anyhow::bail!("cannot use `--ephemeral` with a specified address index");
                }

                let (address, _dtk) = fvk.incoming().ephemeral_address(OsRng);
                println!("{}", address);
            }
        }

        Ok(())
    }
}
