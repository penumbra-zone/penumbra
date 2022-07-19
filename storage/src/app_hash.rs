use ibc::core::ics23_commitment::{commitment::CommitmentPrefix, specs::ProofSpecs};
use jmt::{storage::TreeReader, RootHash};
use once_cell::sync::Lazy;
use penumbra_proto::Message;
use sha2::{Digest, Sha256};

/// this is a proof spec for computing Penumbra's AppHash, which is defined as
/// SHA256("PenumbraAppHash" || jmt.root()). In ICS/IBC terms, this applies a single global prefix
/// to Penumbra's state. Having a stable merkle prefix is currently required for our IBC
/// counterparties to verify our proofs.
fn apphash_spec() -> ics23::ProofSpec {
    ics23::ProofSpec {
        // the leaf hash is simply H(key || value)
        leaf_spec: Some(ics23::LeafOp {
            prefix: vec![],
            hash: ics23::HashOp::Sha256.into(),
            length: ics23::LengthOp::NoPrefix.into(),
            prehash_key: ics23::HashOp::NoHash.into(),
            prehash_value: ics23::HashOp::NoHash.into(),
        }),
        // NOTE: we don't actually use any InnerOps.
        inner_spec: Some(ics23::InnerSpec {
            hash: ics23::HashOp::Sha256.into(),
            child_order: vec![0, 1],
            child_size: 32,
            empty_child: vec![],
            min_prefix_length: 0,
            max_prefix_length: 0,
        }),
        min_depth: 0,
        max_depth: 1,
    }
}

static APPHASH_DOMSEP: &str = "PenumbraAppHash";

pub static PENUMBRA_PROOF_SPECS: Lazy<ProofSpecs> =
    Lazy::new(|| ProofSpecs::from(vec![jmt::ics23_spec(), apphash_spec()]));

pub static PENUMBRA_COMMITMENT_PREFIX: Lazy<CommitmentPrefix> =
    Lazy::new(|| CommitmentPrefix::try_from(APPHASH_DOMSEP.as_bytes().to_vec()).unwrap());

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AppHash(pub [u8; 32]);

// the app hash of penumbra's state is defined as SHA256("PenumbraAppHash" || jmt.root_hash())
impl From<RootHash> for AppHash {
    fn from(r: RootHash) -> Self {
        let mut h = Sha256::new();
        h.update(APPHASH_DOMSEP);
        h.update(r.0);

        AppHash(h.finalize().into())
    }
}

/// given a JMT, a key, and a height, return a tendermint::Proof of the value all the way up to the
/// AppHash.
pub async fn get_with_proof<'a, R: TreeReader>(
    store: &jmt::JellyfishMerkleTree<'a, R>,
    key: Vec<u8>,
    height: u64,
) -> anyhow::Result<(Vec<u8>, tendermint::merkle::proof::Proof)> {
    let jmt_root = store.get_root_hash(height).await?;
    let jmt_proof = store.get_with_ics23_proof(key.clone(), height).await?;
    let value = jmt_proof.value.clone();

    let jmt_commitment_proof = ics23::CommitmentProof {
        proof: Some(ics23::commitment_proof::Proof::Exist(jmt_proof)),
    };

    let root_proof = ics23::CommitmentProof {
        proof: Some(ics23::commitment_proof::Proof::Exist(
            ics23::ExistenceProof {
                key: APPHASH_DOMSEP.into(),
                value: jmt_root.0.to_vec(),
                path: vec![],
                leaf: apphash_spec().leaf_spec,
            },
        )),
    };

    let jmt_op = tendermint::merkle::proof::ProofOp {
        field_type: "jmt:v".to_string(),
        key,
        data: jmt_commitment_proof.encode_to_vec(),
    };

    let root_op = tendermint::merkle::proof::ProofOp {
        field_type: "apphash".to_string(),
        key: APPHASH_DOMSEP.into(),
        data: root_proof.encode_to_vec(),
    };

    Ok((
        value,
        tendermint::merkle::proof::Proof {
            ops: vec![jmt_op, root_op],
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use ibc::core::ics23_commitment::merkle::convert_tm_to_ics_merkle_proof;
    use ibc::core::ics23_commitment::merkle::{apply_prefix, MerkleProof};
    use tempfile::tempdir;

    // simulate a round-trip multiproof verification
    #[tokio::test]
    async fn test_tendermint_multiproof() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("proof-test.db");
        let storage = Storage::load(file_path).await.unwrap();
        let state = storage.state().await.unwrap();

        state.put_proto::<u64>(b"foo-key".into(), 1).await;
        let (jmt_root, height) = state.write().await.commit(storage.clone()).await.unwrap();
        let app_root: AppHash = jmt_root.into();
        let store = jmt::JellyfishMerkleTree::new(&storage);
        let (val2, proof) = get_with_proof(&store, "foo-key".into(), height)
            .await
            .unwrap();

        let ics_merkle: MerkleProof = convert_tm_to_ics_merkle_proof(&proof)
            .expect("couldn't decode tm proof")
            .into();

        let root = ibc_proto::ibc::core::commitment::v1::MerkleRoot {
            hash: app_root.0.to_vec(),
        };

        let merkle_path = apply_prefix(&PENUMBRA_COMMITMENT_PREFIX, vec!["foo-key".to_string()]);

        ics_merkle
            .verify_membership(&PENUMBRA_PROOF_SPECS, root, merkle_path, val2, 0)
            .expect("couldn't verify chained merkle proof");
    }
}
