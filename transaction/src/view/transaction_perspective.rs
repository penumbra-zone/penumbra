use penumbra_crypto::{note, Note, Nullifier, PayloadKey};
use penumbra_proto::core::transaction::v1alpha1::{
    self as pb, NullifierWithNote, PayloadKeyWithCommitment,
};
use std::collections::BTreeMap;

/// This represents the data to understand an individual transaction without
/// disclosing viewing keys.
pub struct TransactionPerspective {
    /// List of per-action payload keys. These can be used to decrypt
    /// the notes, swaps, and memo keys in the transaction.
    ///
    /// One-to-one correspondence between:
    /// * Output and note,
    /// * Swap and note (NFT),
    ///
    /// There is not a one-to-one correspondence between SwapClaim and notes,
    /// i.e. there are two notes per SwapClaim.
    ///
    /// For outputs, we can use the PayloadKey associated with that output
    /// to decrypt the wrapped_memo_key, which will be used to decrypt the
    /// memo in the transaction. This needs to be done only once, because
    /// there is one memo shared between all outputs.
    pub payload_keys: BTreeMap<note::Commitment, PayloadKey>,
    /// Mapping of nullifiers spent in this transaction to notes.
    pub spend_nullifiers: BTreeMap<Nullifier, Option<Note>>,
}

impl TransactionPerspective {}

impl From<TransactionPerspective> for pb::TransactionPerspective {
    fn from(msg: TransactionPerspective) -> Self {
        let mut payload_keys = Vec::new();
        let mut spend_nullifiers = Vec::new();

        for (commitment, payload_key) in msg.payload_keys {
            payload_keys.push(PayloadKeyWithCommitment {
                payload_key: payload_key.to_vec().into(),
                commitment: Some(commitment.to_owned().into()),
            });
        }

        for (nullifier, note) in msg.spend_nullifiers {
            if let Some(note) = note {
                spend_nullifiers.push(NullifierWithNote {
                    nullifier: Some(nullifier.into()),
                    note: Some(note.into()),
                })
            }
        }
        Self {
            payload_keys,
            spend_nullifiers,
        }
    }
}

impl TryFrom<pb::TransactionPerspective> for TransactionPerspective {
    type Error = anyhow::Error;

    fn try_from(msg: pb::TransactionPerspective) -> Result<Self, Self::Error> {
        let mut payload_keys = BTreeMap::new();
        let mut spend_nullifiers = BTreeMap::new();

        for pk in msg.payload_keys {
            if pk.commitment.is_some() {
                payload_keys.insert(
                    pk.commitment.unwrap().try_into()?,
                    <[u8; 32]>::try_from(pk.payload_key.as_ref())?.into(),
                );
            };
        }

        for nwn in msg.spend_nullifiers {
            spend_nullifiers.insert(
                nwn.nullifier.unwrap().try_into()?,
                Some(nwn.note.unwrap().try_into()?),
            );
        }
        Ok(Self {
            payload_keys,
            spend_nullifiers,
        })
    }
}
