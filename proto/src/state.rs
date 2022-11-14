#![cfg(feature = "penumbra-storage")]
mod read;
pub use read::StateReadProto;
mod write;
pub use write::StateWriteProto;
