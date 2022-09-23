use std::collections::BTreeMap;

use penumbra_crypto::note;
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;

#[derive(Clone, Debug)]
pub struct WitnessData {
    pub anchor: tct::Root,
    pub note_commitment_proofs: BTreeMap<note::Commitment, tct::Proof>,
}

impl WitnessData {
    /// Add proof to the existing witness data
    pub fn add_proof(&mut self, nc: note::Commitment, proof: tct::Proof) {
        self.note_commitment_proofs.insert(nc, proof);
    }
}

impl Protobuf<pb::WitnessData> for WitnessData {}

impl From<WitnessData> for pb::WitnessData {
    fn from(msg: WitnessData) -> Self {
        Self {
            anchor: Some(msg.anchor.into()),
            note_commitment_proofs: msg
                .note_commitment_proofs
                .into_iter()
                .map(|(_, v)| v.into())
                .collect(),
        }
    }
}

impl TryFrom<pb::WitnessData> for WitnessData {
    type Error = anyhow::Error;

    fn try_from(msg: pb::WitnessData) -> Result<Self, Self::Error> {
        let mut note_commitment_proofs = BTreeMap::new();
        for proof in msg.note_commitment_proofs {
            let tct_proof: tct::Proof = proof.try_into()?;
            note_commitment_proofs.insert(tct_proof.commitment(), tct_proof);
        }
        Ok(Self {
            anchor: msg
                .anchor
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            note_commitment_proofs,
        })
    }
}
