use ibc_types::core::commitment::MerklePrefix;
use once_cell::sync::Lazy;

/// The substore prefix used for IBC data.
pub static IBC_SUBSTORE_PREFIX: &'static str = "ibc-data";

/// The IBC commitment prefix used for the IBC substore, as a [`MerklePrefix`].
pub static IBC_COMMITMENT_PREFIX: Lazy<MerklePrefix> = Lazy::new(|| MerklePrefix {
    key_prefix: IBC_SUBSTORE_PREFIX.as_bytes().to_vec(),
});

// Vendored from the jmt crate, rather than importing it from e.g., cnidarium
// The proof specs need to be available for use cases like relayers, so it should be
// outside of the feature-gated component implementation, and we only depend on
// cnidarium when actually implementing the component (since that pulls in all of rocksdb, etc).
// Vendoring isn't ideal, but this data is effectively "vendored" anyways since it needs to
// be replicated across relayers, chain configs, ....
mod vendored {
    const LEAF_DOMAIN_SEPARATOR: &[u8] = b"JMT::LeafNode";
    const INTERNAL_DOMAIN_SEPARATOR: &[u8] = b"JMT::IntrnalNode";

    const SPARSE_MERKLE_PLACEHOLDER_HASH: [u8; 32] = *b"SPARSE_MERKLE_PLACEHOLDER_HASH__";

    pub fn ics23_spec() -> ics23::ProofSpec {
        ics23::ProofSpec {
            leaf_spec: Some(ics23::LeafOp {
                hash: ics23::HashOp::Sha256.into(),
                prehash_key: ics23::HashOp::Sha256.into(),
                prehash_value: ics23::HashOp::Sha256.into(),
                length: ics23::LengthOp::NoPrefix.into(),
                prefix: LEAF_DOMAIN_SEPARATOR.to_vec(),
            }),
            inner_spec: Some(ics23::InnerSpec {
                hash: ics23::HashOp::Sha256.into(),
                child_order: vec![0, 1],
                min_prefix_length: INTERNAL_DOMAIN_SEPARATOR.len() as i32,
                max_prefix_length: INTERNAL_DOMAIN_SEPARATOR.len() as i32,
                child_size: 32,
                empty_child: SPARSE_MERKLE_PLACEHOLDER_HASH.to_vec(),
            }),
            min_depth: 0,
            max_depth: 64,
            prehash_key_before_comparison: true,
        }
    }
}

/// The ICS23 proof spec for penumbra's IBC state; this can be used to verify proofs
/// for other substores in the penumbra state, provided that the data is indeed inside a substore
/// (as opposed to directly in the root store.)
pub static IBC_PROOF_SPECS: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![vendored::ics23_spec(), vendored::ics23_spec()]);

/// TODO: upstream into ibc-types
pub trait MerklePrefixExt {
    fn apply_string(&self, path: String) -> String;
}

impl MerklePrefixExt for MerklePrefix {
    fn apply_string(&self, path: String) -> String {
        let prefix_string = String::from_utf8(self.key_prefix.clone())
            .expect("commitment prefix is not valid utf-8");

        format!("{}/{}", prefix_string, path)
    }
}
