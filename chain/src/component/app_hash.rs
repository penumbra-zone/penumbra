use anyhow::anyhow;
use async_trait::async_trait;
use ibc_types::core::ics23_commitment::merkle::MerkleProof;
use ibc_types::core::ics23_commitment::{commitment::CommitmentPrefix, specs::ProofSpecs};
use once_cell::sync::Lazy;
use penumbra_proto::Message;
use penumbra_storage::{RootHash, Snapshot};

use sha2::{Digest, Sha256};

pub static PENUMBRA_PROOF_SPECS: Lazy<ProofSpecs> =
    Lazy::new(|| ProofSpecs::from(vec![penumbra_storage::ics23_spec(), apphash_spec()]));

pub static PENUMBRA_COMMITMENT_PREFIX: Lazy<CommitmentPrefix> =
    Lazy::new(|| CommitmentPrefix::try_from(APPHASH_DOMSEP.as_bytes().to_vec()).unwrap());

static APPHASH_DOMSEP: &str = "PenumbraAppHash";

#[derive(Copy, Clone, PartialEq, Eq)]
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

impl std::fmt::Debug for AppHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AppHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}
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
        prehash_key_before_comparison: true,
    }
}

#[async_trait]
pub trait AppHashRead {
    async fn get_with_proof_to_apphash(
        &self,
        key: Vec<u8>,
    ) -> Result<(Vec<u8>, MerkleProof), anyhow::Error>;
    async fn get_with_proof_to_apphash_tm(
        &self,
        key: Vec<u8>,
    ) -> Result<(Vec<u8>, tendermint::merkle::proof::ProofOps), anyhow::Error>;
    async fn app_hash(&self) -> Result<AppHash, anyhow::Error>;
}

#[async_trait]
impl AppHashRead for Snapshot {
    async fn app_hash(&self) -> anyhow::Result<AppHash> {
        let root = self.root_hash().await?;
        Ok(AppHash::from(root))
    }

    async fn get_with_proof_to_apphash(
        &self,
        key: Vec<u8>,
    ) -> anyhow::Result<(Vec<u8>, MerkleProof)> {
        let (value, jmt_proof) = self.get_with_proof(key.clone()).await?;

        // TODO(erwan): will immediately follow-up this pr with one that changes
        // signature to Option<Vec<u8>> here as well, and a `get_without_proof`.
        // For now, we conserve the semantics of error-ing out on a missing key.
        let value = value.ok_or_else(|| anyhow!("key not found"))?;

        let jmt_root = self.root_hash().await?;

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

        Ok((
            value,
            MerkleProof {
                proofs: vec![jmt_proof, root_proof],
            },
        ))
    }

    async fn get_with_proof_to_apphash_tm(
        &self,
        key: Vec<u8>,
    ) -> Result<(Vec<u8>, tendermint::merkle::proof::ProofOps), anyhow::Error> {
        let (value, proof_ics) = self.get_with_proof_to_apphash(key.to_vec()).await?;

        let jmt_op = tendermint::merkle::proof::ProofOp {
            field_type: "jmt:v".to_string(),
            key,
            data: proof_ics.proofs[0].encode_to_vec(),
        };
        let root_op = tendermint::merkle::proof::ProofOp {
            field_type: "apphash".to_string(),
            key: APPHASH_DOMSEP.into(),
            data: proof_ics.proofs[1].encode_to_vec(),
        };

        Ok((
            value,
            tendermint::merkle::proof::ProofOps {
                ops: vec![jmt_op, root_op],
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    /*
    use super::super::*;
    use super::*;
    use ibc_types::core::ics23_commitment::merkle::convert_tm_to_ics_merkle_proof;
    use ibc_types::core::ics23_commitment::merkle::{apply_prefix, MerkleProof};
    use tempfile::tempdir;

    // simulate a round-trip multiproof verification

    #[tokio::test]
    async fn test_tendermint_multiproof() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("proof-test.db");
        let storage = Storage::load(file_path.clone()).await.unwrap();
        let mut state = storage.latest_state();
        let mut tx = state.begin_transaction();

        tx.put_proto::<u64>("foo-key".into(), 1);
        tx.apply();
        let jmt_root = storage.clone().commit(state).await.unwrap();
        let app_root: AppHash = jmt_root.into();

        let state = storage.latest_state();
        let (val2, proof) = get_with_proof(&state, "foo-key".into(), &jmt_root)
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
    */
}
