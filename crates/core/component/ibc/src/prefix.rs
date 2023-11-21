use ibc_types::core::commitment::MerklePrefix;
use once_cell::sync::Lazy;

/// The substore prefix used for IBC data.
pub static IBC_SUBSTORE_PREFIX: &'static str = "ibc-data/";

/// The IBC commitment prefix used for the IBC substore, as a [`MerklePrefix`].
pub static IBC_COMMITMENT_PREFIX: Lazy<MerklePrefix> = Lazy::new(|| MerklePrefix {
    key_prefix: IBC_SUBSTORE_PREFIX.as_bytes().to_vec(),
});

/// the ICS23 proof spec for penumbra's IBC state; this can be used to verify proofs
/// for other substores in the penumbra state, provided that the data is indeed inside a substore
/// (as opposed to directly in the root store.)
pub static IBC_PROOF_SPECS: Lazy<Vec<ics23::ProofSpec>> = Lazy::new(|| {
    vec![
        penumbra_storage::ics23_spec(),
        penumbra_storage::ics23_spec(),
    ]
});

/// TODO: upstream into ibc-types
pub trait MerklePrefixExt {
    fn apply_string(&self, path: String) -> String;
}

impl MerklePrefixExt for MerklePrefix {
    fn apply_string(&self, path: String) -> String {
        let prefix_string = String::from_utf8(self.key_prefix.clone())
            .expect("commitment prefix is not valid utf-8");

        format!("{}{}", prefix_string, path)
    }
}
