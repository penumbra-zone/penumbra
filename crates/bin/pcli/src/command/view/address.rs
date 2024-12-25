use anyhow::Result;
use base64::Engine;
use rand_core::OsRng;
use std::str::FromStr;

use penumbra_sdk_keys::{keys::AddressIndex, Address, FullViewingKey};

#[derive(Debug, clap::Parser)]
pub struct AddressCmd {
    /// The address to provide information about
    #[clap(default_value = "0")]
    address_or_index: String,
    /// Generate an ephemeral address instead of an indexed one.
    #[clap(short, long)]
    ephemeral: bool,
    /// Output in base64 format, instead of the default bech32.
    #[clap(long)]
    base64: bool,
    /// Use transparent (bech32, 32-byte) address encoding, for compatibility with some IBC chains.
    #[clap(long)]
    transparent: bool,
    /// Print the current FVK
    #[clap(long)]
    fvk: bool,
    /// Generate a payment address from a provided full viewing key
    #[clap(long)]
    from_fvk: Option<String>,
}

impl AddressCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        let index: Result<u32, _> = self.address_or_index.parse();

        if let Ok(index) = index {
            //index provided

            let (address, _dtk) = match self.ephemeral {
                false => fvk.incoming().payment_address(index.into()),
                true => fvk.incoming().ephemeral_address(OsRng, index.into()),
            };

            if self.base64 {
                println!(
                    "{}",
                    base64::engine::general_purpose::STANDARD.encode(address.to_vec()),
                );
            } else if self.transparent {
                if index != 0 {
                    return Err(anyhow::anyhow!(
                        "warning: index must be 0 to use transparent address encoding"
                    ));
                }
                println!("{}", fvk.incoming().transparent_address());
            } else {
                if self.fvk {
                    eprintln!("🔥 CAUTION: POSSESSION OF THE FOLLOWING FULL VIEWING KEY WILL");
                    eprintln!("🔥 PROVIDE VISIBILITY TO ALL ACTIVITY ON ITS ASSOCIATED ACCOUNTS.");
                    eprintln!("🔥 DISTRIBUTE WITH CARE!");
                    eprintln!("");
                    println!("{}", fvk);
                } else if let Some(fvk) = &self.from_fvk {
                    let (address, _) = FullViewingKey::payment_address(
                        &FullViewingKey::from_str(&fvk[..])?,
                        AddressIndex::new(0),
                    );

                    println!("{}", address);
                } else {
                    println!("{}", address);
                }
            };
        } else {
            //address or nothing provided

            let address: Address = self
                .address_or_index
                .parse()
                .map_err(|_| anyhow::anyhow!("Provided address is invalid."))?;

            match fvk.address_index(&address) {
                Some(address_index) => println!(
                    "Address is viewable with this full viewing key. Account index is {0}. {1}",
                    address_index.account,
                    match address_index.randomizer != [0u8; 12] {
                        true => "Address is an IBC deposit address.",
                        false => "",
                    }
                ),
                None => println!("Address is not viewable with this full viewing key."),
            }
        }

        Ok(())
    }
}
