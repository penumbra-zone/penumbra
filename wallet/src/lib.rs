// Required because of NCT type size
#![recursion_limit = "256"]

mod state;
mod wallet;

pub use state::{ClientState, UnspentNote};
pub use wallet::Wallet;
