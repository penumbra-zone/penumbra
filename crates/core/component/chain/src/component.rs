mod view;

pub mod rpc;

use ibc_types::core::commitment::MerklePrefix;
use once_cell::sync::Lazy;

// the commitment prefix for the IBC state in the penumbra chain.
pub static PENUMBRA_IBC_COMMITMENT_PREFIX: Lazy<MerklePrefix> = Lazy::new(|| MerklePrefix {
    key_prefix: "ibc/".as_bytes().to_vec(),
});

// the ICS23 proof spec for penumbra's IBC state; this can be used to verify proofs
// for other substores in the penumbra state, provided that the data is indeed inside a substore
// (as opposed to directly in the root store.)
pub static PENUMBRA_IBC_PROOF_SPECS: Lazy<Vec<ics23::ProofSpec>> = Lazy::new(|| {
    vec![
        penumbra_storage::ics23_spec(),
        penumbra_storage::ics23_spec(),
    ]
});
pub use view::{StateReadExt, StateWriteExt};
