use decaf377_fmd as fmd;
use decaf377_ka as ka;
use decaf377_rdsa as rdsa;

pub mod address;
pub mod keys;
pub mod prf;

pub use address::{Address, AddressVar, AddressView};
pub use keys::FullViewingKey;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}
