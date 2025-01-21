use anyhow::Result;
use rand_core::OsRng;

use penumbra_sdk_keys::{Address, FullViewingKey};

#[derive(Debug, clap::Parser)]
pub struct NobleAddressCmd {
    /// The address to provide information about
    #[clap(default_value = "0")]
    address_or_index: String,
    /// Generate an ephemeral address instead of an indexed one.
    #[clap(short, long)]
    ephemeral: bool,
    /// The Noble IBC channel to use for forwarding.
    #[clap(long)]
    channel: String,
}

impl NobleAddressCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        let index: Result<u32, _> = self.address_or_index.parse();

        let address = if let Ok(index) = index {
            // address index provided
            let (address, _dtk) = match self.ephemeral {
                false => fvk.incoming().payment_address(index.into()),
                true => fvk.incoming().ephemeral_address(OsRng, index.into()),
            };

            address
        } else {
            // address or nothing provided
            let address: Address = self
                .address_or_index
                .parse()
                .map_err(|_| anyhow::anyhow!("Provided address is invalid."))?;

            address
        };

        let noble_address = address.noble_forwarding_address(&self.channel);

        println!("{}", noble_address);

        Ok(())
    }
}
