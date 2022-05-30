use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::FullViewingKey;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum AddrCmd {
    /// Show the address with the given index.
    Show {
        /// The index of the address to show.
        index: u64,
        /// If true, emits only the address and not the (local) label for it.
        #[structopt(short, long)]
        addr_only: bool,
    },
}

impl AddrCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            AddrCmd::Show { .. } => false,
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
        }

        // Print the table (we don't get here if `show --addr-only`)
        println!("{}", table);

        Ok(())
    }
}
