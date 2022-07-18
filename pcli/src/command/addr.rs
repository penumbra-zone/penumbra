use anyhow::Result;
use comfy_table::{presets, Table};
use rand_core::OsRng;

use penumbra_crypto::FullViewingKey;

#[derive(Debug, clap::Subcommand)]
pub enum AddrCmd {
    /// Show the address with the given index.
    Show {
        /// The index of the address to show.
        /// Default to 0
        #[clap(default_value = "0")]
        index: u64,
        /// If true, emits only the address and not the (local) label for it.
        #[clap(short, long)]
        addr_only: bool,
    },
    /// Generates an ephemeral address and prints it.
    Ephemeral,
}

impl AddrCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            AddrCmd::Show { .. } => false,
            AddrCmd::Ephemeral => false,
        }
    }

    pub fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        // Set up table (this won't be used with `show --addr-only`)
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(vec!["Index", "Address"]);

        match self {
            AddrCmd::Show { index, addr_only } => {
                let (address, _dtk) = fvk.incoming().payment_address((*index).into());

                if *addr_only {
                    println!("{}", address);
                    return Ok(()); // don't print the label
                } else {
                    table.add_row(vec![index.to_string(), address.to_string()]);
                }
            }
            AddrCmd::Ephemeral => {
                let (address, _dtk) = fvk.incoming().ephemeral_address(OsRng);
                println!("{}", address);
                return Ok(());
            }
        }

        // Print the table (we don't get here if `show --addr-only`)
        println!("{}", table);

        Ok(())
    }
}
