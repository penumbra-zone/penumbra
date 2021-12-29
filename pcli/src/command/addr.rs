use structopt::StructOpt;

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
}
