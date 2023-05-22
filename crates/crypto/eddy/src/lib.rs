//! An implementation of [flow encryption][protocol-batching].
//!
//! Current status:
//! - [x] Encryption
//! - [x] Decryption
//! - [x] Decryption Proofs
//! - [x] Lookup table interface
//! - [ ] Error on insufficient shares
//! - [ ] Distributed key generation
//! - [ ] Serialization
//! - [ ] Encryption Proofs
//!
//! [protocol-batching]: https://protocol.penumbra.zone/main/concepts/batching_flows.html

mod ciphertext;
mod decryption_share;
mod decryption_table;
mod encryption_key;
mod key_share;
mod limb;
mod transcript;
mod value;

use transcript::TranscriptProtocol;

pub mod dkg;
pub mod proofs;

pub use ciphertext::{Ciphertext, InsufficientSharesError};
pub use decryption_share::{DecryptionShare, Unverified, VerificationStatus, Verified};
pub use decryption_table::{DecryptionTable, MockDecryptionTable, TableLookupError};
pub use encryption_key::EncryptionKey;
pub use key_share::{PrivateKeyShare, PublicKeyShare};
pub use value::Value;
