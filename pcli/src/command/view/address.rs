use anyhow::Result;
use base64::Engine;
use rand_core::OsRng;

use penumbra_crypto::FullViewingKey;

#[derive(Debug, clap::Parser)]
pub struct AddressCmd {
    /// Show the address with a particular numerical index [default: 0].
    index: u32,
    /// Generate an ephemeral address instead of an indexed one.
    #[clap(short, long)]
    ephemeral: bool,
    /// Output in base64 format, instead of the default bech32.
    #[clap(long)]
    base64: bool,
}

impl AddressCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        let (address, _dtk) = match self.ephemeral {
            false => fvk.incoming().payment_address(self.index.into()),
            true => fvk.incoming().ephemeral_address(OsRng, self.index.into()),
        };

        if self.base64 {
            println!(
                "{}",
                base64::engine::general_purpose::STANDARD.encode(address.to_vec())
            );
        } else {
            println!("{}", address);
        }

        Ok(())
    }
}
