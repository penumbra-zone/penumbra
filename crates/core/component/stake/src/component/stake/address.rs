use {
    sha2::{Digest, Sha256},
    tendermint::PublicKey,
};

/// A type alias for 20-byte truncated SHA256 validator addresses.
///
/// This is the format in which [`tendermint::abci::types::CommitInfo`] presents vote information.
pub(crate) type Address = [u8; ADDRESS_LEN];

const ADDRESS_LEN: usize = 20;

/// Translates from consensus keys to the truncated sha256 hashes in `last_commit_info`.
//
//  NOTE: This should really be a refined type upstream, but we can't currently upstream to
//  tendermint-rs, for process reasons, and shouldn't do our own tendermint data modeling, so
//  this is an interim hack.
pub(crate) fn validator_address(ck: &PublicKey) -> Address {
    let ck_bytes = ck.to_bytes();
    Sha256::digest(ck_bytes).as_slice()[0..ADDRESS_LEN]
        .try_into()
        .expect("Sha256 digest should be 20-bytes long")
}
