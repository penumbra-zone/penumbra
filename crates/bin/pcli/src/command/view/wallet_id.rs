use anyhow::Result;

use penumbra_sdk_keys::FullViewingKey;

#[derive(Debug, clap::Parser)]
pub struct WalletIdCmd {}

impl WalletIdCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        let wallet_id = fvk.wallet_id();
        println!("{wallet_id}");

        Ok(())
    }
}
