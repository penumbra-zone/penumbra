use penumbra_crypto::FieldExt;
use serde_with::serde_as;

use super::*;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ClientStateHelper {
    last_block_height: Option<u32>,
    #[serde_as(as = "serde_with::hex::Hex")]
    note_commitment_tree: Vec<u8>,
    nullifier_map: Vec<(String, String)>,
    unspent_set: Vec<(String, String)>,
    #[serde(default)]
    pending_set: Vec<(String, SystemTime, String)>,
    #[serde(default)]
    pending_change_set: Vec<(String, SystemTime, String)>,
    spent_set: Vec<(String, String)>,
    transactions: Vec<(String, String)>,
    asset_registry: Vec<(String, String)>,
    wallet: Wallet,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub enum PendingNoteCommitmentHelper {
    #[serde_as(as = "serde_with::hex::Hex")]
    Change(String),
    #[serde_as(as = "serde_with::hex::Hex")]
    Spend(String),
}

impl From<PendingNoteCommitment> for PendingNoteCommitmentHelper {
    fn from(pending_note_commitment: PendingNoteCommitment) -> Self {
        match pending_note_commitment {
            PendingNoteCommitment::Change(commitment) => {
                PendingNoteCommitmentHelper::Change(hex::encode(commitment.0.to_bytes()))
            }
            PendingNoteCommitment::Spend(commitment) => {
                PendingNoteCommitmentHelper::Spend(hex::encode(commitment.0.to_bytes()))
            }
        }
    }
}

impl TryFrom<PendingNoteCommitmentHelper> for PendingNoteCommitment {
    type Error = anyhow::Error;

    fn try_from(
        pending_note_commitment: PendingNoteCommitmentHelper,
    ) -> Result<Self, anyhow::Error> {
        Ok(match pending_note_commitment {
            PendingNoteCommitmentHelper::Change(commitment) => {
                let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                PendingNoteCommitment::Change(commitment)
            }
            PendingNoteCommitmentHelper::Spend(commitment) => {
                let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                PendingNoteCommitment::Spend(commitment)
            }
        })
    }
}

impl From<ClientState> for ClientStateHelper {
    fn from(state: ClientState) -> Self {
        Self {
            wallet: state.wallet,
            last_block_height: state.last_block_height,
            note_commitment_tree: bincode::serialize(&state.note_commitment_tree).unwrap(),
            nullifier_map: state
                .nullifier_map
                .iter()
                .map(|(nullifier, commitment)| {
                    (
                        hex::encode(nullifier.0.to_bytes()),
                        hex::encode(commitment.0.to_bytes()),
                    )
                })
                .collect(),
            unspent_set: state
                .unspent_set
                .iter()
                .map(|(commitment, note)| {
                    (
                        hex::encode(commitment.0.to_bytes()),
                        hex::encode(note.to_bytes()),
                    )
                })
                .collect(),
            pending_set: state
                .pending_set
                .iter()
                .map(|(commitment, (timeout, note))| {
                    (
                        hex::encode(commitment.0.to_bytes()),
                        *timeout,
                        hex::encode(note.to_bytes()),
                    )
                })
                .collect(),
            pending_change_set: state
                .pending_change_set
                .iter()
                .map(|(commitment, (timeout, note))| {
                    (
                        hex::encode(commitment.0.to_bytes()),
                        *timeout,
                        hex::encode(note.to_bytes()),
                    )
                })
                .collect(),
            spent_set: state
                .spent_set
                .iter()
                .map(|(commitment, note)| {
                    (
                        hex::encode(commitment.0.to_bytes()),
                        hex::encode(note.to_bytes()),
                    )
                })
                .collect(),
            asset_registry: state
                .asset_cache
                .iter()
                .map(|(id, denom)| (hex::encode(id.to_bytes()), denom.to_string()))
                .collect(),
            // TODO: serialize full transactions
            transactions: vec![],
        }
    }
}

impl TryFrom<ClientStateHelper> for ClientState {
    type Error = anyhow::Error;

    fn try_from(state: ClientStateHelper) -> Result<Self, Self::Error> {
        let mut nullifier_map = BTreeMap::new();

        for (nullifier, commitment) in state.nullifier_map.into_iter() {
            nullifier_map.insert(
                hex::decode(nullifier)?.as_slice().try_into()?,
                hex::decode(commitment)?.as_slice().try_into()?,
            );
        }

        let mut unspent_set = BTreeMap::new();
        for (commitment, note) in state.unspent_set.into_iter() {
            unspent_set.insert(
                hex::decode(commitment)?.as_slice().try_into()?,
                hex::decode(note)?.as_slice().try_into()?,
            );
        }

        let mut pending_set = BTreeMap::new();
        for (commitment, timeout, note) in state.pending_set.into_iter() {
            pending_set.insert(
                hex::decode(commitment)?.as_slice().try_into()?,
                (timeout, hex::decode(note)?.as_slice().try_into()?),
            );
        }

        let mut pending_change_set = BTreeMap::new();
        for (commitment, timeout, note) in state.pending_change_set.into_iter() {
            pending_change_set.insert(
                hex::decode(commitment)?.as_slice().try_into()?,
                (timeout, hex::decode(note)?.as_slice().try_into()?),
            );
        }

        let mut spent_set = BTreeMap::new();
        for (commitment, note) in state.spent_set.into_iter() {
            spent_set.insert(
                hex::decode(commitment)?.as_slice().try_into()?,
                hex::decode(note)?.as_slice().try_into()?,
            );
        }

        let mut asset_registry = BTreeMap::new();
        for (id, denom) in state.asset_registry.into_iter() {
            asset_registry.insert(hex::decode(id)?.try_into()?, denom);
        }

        Ok(Self {
            wallet: state.wallet,
            last_block_height: state.last_block_height,
            note_commitment_tree: bincode::deserialize(&state.note_commitment_tree)?,
            nullifier_map,
            unspent_set,
            pending_set,
            pending_change_set,
            spent_set,
            asset_cache: asset_registry.try_into()?,
            // TODO: serialize full transactions
            transactions: Default::default(),
        })
    }
}
