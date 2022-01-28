use anyhow::Result;
use comfy_table::{presets, Table};
use structopt::StructOpt;

use crate::ClientStateFile;

#[derive(Debug, StructOpt)]
pub enum AddrCmd {
    /// List addresses.
    List,
    /// Show the address with the given index.
    Show {
        /// The index of the address to show.
        #[structopt(short, long)]
        index: u32,
        /// If true, emits only the address and not the (local) label for it.
        #[structopt(short, long)]
        addr_only: bool,
    },
    /// Create a new address.
    New {
        /// A freeform label for the address, stored only locally.
        label: String,
    },
}

impl AddrCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            AddrCmd::List => false,
            AddrCmd::Show { .. } => false,
            AddrCmd::New { .. } => false,
        }
    }

    pub fn exec(&self, state: &mut ClientStateFile) -> Result<()> {
        // Set up table (this won't be used with `show --addr-only`)
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(vec!["Index", "Label", "Address"]);

        match self {
            AddrCmd::List => {
                for (index, label, address) in state.wallet().addresses() {
                    table.add_row(vec![index.to_string(), label, address.to_string()]);
                }
            }
            AddrCmd::Show { index, addr_only } => {
                let (label, address) = state.wallet().address_by_index(*index as u64)?;

                if *addr_only {
                    println!("{}", address.to_string());
                    return Ok(()); // don't print the label
                } else {
                    table.add_row(vec![index.to_string(), label, address.to_string()]);
                }
            }
            AddrCmd::New { label } => {
                let (index, address, _dtk) = state.wallet_mut().new_address(label.clone());
                state.commit()?;
                table.add_row(vec![index.to_string(), label.clone(), address.to_string()]);
            }
        }

        // Print the table (we don't get here if `show --addr-only`)
        println!("{}", table);

        Ok(())
    }
}
