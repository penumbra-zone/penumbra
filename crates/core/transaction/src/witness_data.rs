use std::collections::BTreeMap;

use penumbra_crypto::note;
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;

#[derive(Clone, Debug)]
pub struct WitnessData {
    pub anchor: tct::Root,
    pub state_commitment_proofs: BTreeMap<note::StateCommitment, tct::Proof>,
}

impl WitnessData {
    /// Add proof to the existing witness data
    pub fn add_proof(&mut self, nc: note::StateCommitment, proof: tct::Proof) {
        self.state_commitment_proofs.insert(nc, proof);
    }
}

impl TypeUrl for WitnessData {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.WitnessData";
}

impl DomainType for WitnessData {
    type Proto = pb::WitnessData;
}

impl From<WitnessData> for pb::WitnessData {
    fn from(msg: WitnessData) -> Self {
        Self {
            anchor: Some(msg.anchor.into()),
            state_commitment_proofs: msg
                .state_commitment_proofs
                .into_values()
                .map(|v| v.into())
                .collect(),
        }
    }
}

impl TryFrom<pb::WitnessData> for WitnessData {
    type Error = anyhow::Error;

    fn try_from(msg: pb::WitnessData) -> Result<Self, Self::Error> {
        let mut state_commitment_proofs = BTreeMap::new();
        for proof in msg.state_commitment_proofs {
            let tct_proof: tct::Proof = proof.try_into()?;
            state_commitment_proofs.insert(tct_proof.commitment(), tct_proof);
        }
        Ok(Self {
            anchor: msg
                .anchor
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            state_commitment_proofs,
        })
    }
}
