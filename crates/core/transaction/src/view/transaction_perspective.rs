use anyhow::anyhow;
use pbjson_types::Any;
use penumbra_asset::{asset, EstimatedPrice, Value, ValueView};
use penumbra_keys::{Address, AddressView, PayloadKey};
use penumbra_proto::core::transaction::v1::{
    self as pb, NullifierWithNote, PayloadKeyWithCommitment,
};
use penumbra_proto::core::txhash::v1::TransactionId;
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note, NoteView};

use std::collections::{BTreeMap, HashMap};

/// This represents the data to understand an individual transaction without
/// disclosing viewing keys.
#[derive(Debug, Clone, Default)]
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
    pub payload_keys: BTreeMap<note::StateCommitment, PayloadKey>,
    /// Mapping of nullifiers spent in this transaction to notes.
    pub spend_nullifiers: BTreeMap<Nullifier, Note>,
    /// The openings of note commitments referred to in the transaction but otherwise not included in the transaction.
    pub advice_notes: BTreeMap<note::StateCommitment, Note>,
    /// The views of any relevant address.
    pub address_views: Vec<AddressView>,
    /// Any relevant denoms for viewed assets.
    pub denoms: asset::Cache,
    /// The transaction ID associated with this TransactionPerspective
    pub transaction_id: TransactionId,
    // A map of base64-encoded nullfiers to the IDs of the transactions
    // containing the outputs that they nullify.
    pub transaction_ids_by_nullifier: HashMap<String, TransactionId>,
    /// Any relevant estimated prices.
    pub prices: Vec<EstimatedPrice>,
    /// Any relevant extended metadata.
    pub extended_metadata: BTreeMap<asset::Id, Any>,
}

impl TransactionPerspective {
    pub fn view_value(&self, value: Value) -> ValueView {
        value
            .view_with_cache(&self.denoms)
            .with_prices(&self.prices, &self.denoms)
            .with_extended_metadata(self.extended_metadata.get(&value.asset_id).cloned())
    }

    pub fn view_note(&self, note: Note) -> NoteView {
        NoteView {
            address: self.view_address(note.address()),
            value: self.view_value(note.value()),
            rseed: note.rseed(),
        }
    }

    pub fn view_address(&self, address: Address) -> AddressView {
        match self.address_views.iter().find(|av| av.address() == address) {
            Some(av) => av.clone(),
            None => AddressView::Opaque { address },
        }
    }
}

impl TransactionPerspective {}

impl From<TransactionPerspective> for pb::TransactionPerspective {
    fn from(msg: TransactionPerspective) -> Self {
        let mut payload_keys = Vec::new();
        let mut spend_nullifiers = Vec::new();
        let mut advice_notes = Vec::new();
        let mut address_views = Vec::new();
        let mut denoms = Vec::new();

        for (commitment, payload_key) in msg.payload_keys {
            payload_keys.push(PayloadKeyWithCommitment {
                payload_key: Some(payload_key.to_owned().into()),
                commitment: Some(commitment.to_owned().into()),
            });
        }

        for (nullifier, note) in msg.spend_nullifiers {
            spend_nullifiers.push(NullifierWithNote {
                nullifier: Some(nullifier.into()),
                note: Some(note.into()),
            })
        }
        for note in msg.advice_notes.into_values() {
            advice_notes.push(note.into());
        }
        for address_view in msg.address_views {
            address_views.push(address_view.into());
        }
        for denom in msg.denoms.values() {
            denoms.push(denom.clone().into());
        }

        Self {
            payload_keys,
            spend_nullifiers,
            advice_notes,
            address_views,
            denoms,
            transaction_id: Some(msg.transaction_id.into()),
            transaction_ids_by_nullifier: HashMap::new(),
            prices: msg.prices.into_iter().map(Into::into).collect(),
            extended_metadata: msg
                .extended_metadata
                .into_iter()
                .map(|(k, v)| pb::transaction_perspective::ExtendedMetadataById {
                    asset_id: Some(k.into()),
                    extended_metadata: Some(v),
                })
                .collect(),
        }
    }
}

impl TryFrom<pb::TransactionPerspective> for TransactionPerspective {
    type Error = anyhow::Error;

    fn try_from(msg: pb::TransactionPerspective) -> Result<Self, Self::Error> {
        let mut payload_keys = BTreeMap::new();
        let mut spend_nullifiers = BTreeMap::new();
        let mut advice_notes = BTreeMap::new();
        let mut address_views = Vec::new();
        let mut denoms = BTreeMap::new();

        for pk in msg.payload_keys {
            if pk.commitment.is_some() {
                payload_keys.insert(
                    pk.commitment
                        .ok_or_else(|| anyhow!("missing commitment in payload key"))?
                        .try_into()?,
                    pk.payload_key
                        .ok_or_else(|| anyhow!("missing payload key"))?
                        .try_into()?,
                );
            };
        }

        for nwn in msg.spend_nullifiers {
            spend_nullifiers.insert(
                nwn.nullifier
                    .ok_or_else(|| anyhow!("missing nullifier in spend nullifier"))?
                    .try_into()?,
                nwn.note
                    .ok_or_else(|| anyhow!("missing note in spend nullifier"))?
                    .try_into()?,
            );
        }

        for note in msg.advice_notes {
            let note: Note = note.try_into()?;
            advice_notes.insert(note.commit(), note);
        }

        for address_view in msg.address_views {
            address_views.push(address_view.try_into()?);
        }

        for denom in msg.denoms {
            denoms.insert(
                denom
                    .penumbra_asset_id
                    .clone()
                    .ok_or_else(|| anyhow!("missing penumbra asset ID in denom"))?
                    .try_into()?,
                denom.try_into()?,
            );
        }

        let transaction_id: TransactionId = match msg.transaction_id {
            Some(tx_id) => tx_id.try_into()?,
            None => TransactionId::default(),
        };

        Ok(Self {
            payload_keys,
            spend_nullifiers,
            advice_notes,
            address_views,
            denoms: denoms.try_into()?,
            transaction_id,
            transaction_ids_by_nullifier: msg.transaction_ids_by_nullifier,
            prices: msg
                .prices
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            extended_metadata: msg
                .extended_metadata
                .into_iter()
                .map(|em| {
                    Ok((
                        em.asset_id
                            .ok_or_else(|| anyhow!("missing asset ID in extended metadata"))?
                            .try_into()?,
                        em.extended_metadata
                            .ok_or_else(|| anyhow!("missing extended metadata"))?,
                    ))
                })
                .collect::<Result<_, anyhow::Error>>()?,
        })
    }
}
