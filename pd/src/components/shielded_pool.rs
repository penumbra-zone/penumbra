use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::{FieldExt, Fq, Fr};
use penumbra_crypto::{
    asset,
    asset::{Denom, Id},
    ka,
    merkle::{self, Frontier, NoteCommitmentTree, TreeExt},
    Address, Note, One, Value,
};
use penumbra_proto::light_wallet::{CompactBlock, StateFragment};
use penumbra_transaction::{Action, Transaction};
use tendermint::abci;
use tracing::instrument;

use super::{Component, Overlay};
use crate::{genesis, WriteOverlayExt};

// Stub component
pub struct ShieldedPool {
    overlay: Overlay,
    note_commitment_tree: NoteCommitmentTree,
    // TODO: change the on-chain registry to just store asset id -> amount
    // and asset id -> asset id separately
    supply_updates: BTreeMap<asset::Id, i64>,
    new_denoms: BTreeMap<asset::Id, Denom>,
    /// The in-progress CompactBlock representation of the ShieldedPool changes
    compact_block: CompactBlock,
}

#[async_trait]
impl Component for ShieldedPool {
    async fn new(overlay: Overlay) -> Result<Self> {
        let note_commitment_tree = Self::get_nct(&overlay).await?;

        Ok(Self {
            overlay,
            note_commitment_tree,
            supply_updates: Default::default(),
            new_denoms: Default::default(),
            compact_block: Default::default(),
        })
    }

    fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            if allocation.amount == 0 {
                return Err(anyhow!(
                    "Genesis allocations contain empty note: {:?}",
                    allocation
                ));
            }

            let base_denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .ok_or_else(|| {
                    anyhow!(
                        "Genesis denomination {} is not a base denom",
                        allocation.denom
                    )
                })?;

            // A transaction ID is either a hash of a transaction, or special data.
            // Special data is encoded with 23 leading 0 bytes, followed by a nonzero code byte,
            // followed by 8 data bytes.
            //
            // Transaction hashes can be confused with special data only if the transaction hash begins with 23 leading 0 bytes; this happens with probability 2^{-184}.
            //
            // Genesis transaction IDs use code 0x1.
            // TODO: extract into common code
            let transaction_id = [&[0; 23][..], &[1][..], &[0; 8][..]]
                .concat()
                .try_into()
                .unwrap();

            self.mint_note(
                Value {
                    amount: allocation.amount,
                    asset_id: base_denom.id(),
                },
                &allocation.address,
                transaction_id,
            )?;
        }

        self.compact_block.height = 0;
        self.write_block()?;

        Ok(())
    }

    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        Ok(())
    }

    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        todo!()
    }

    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        // TODO: rename transaction_body.merkle_root now that we have 2 merkle trees
        self.check_claimed_anchor(&tx.transaction_body.merkle_root)
            .await?;

        todo!()
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        let should_quarantine = tx
            .transaction_body
            .actions
            .iter()
            .any(|action| matches!(action, Action::Undelegate { .. }));

        let id = tx.id();
        let fragments = tx
            .transaction_body
            .actions
            .iter()
            .filter_map(|action| {
                if let Action::Output(output) = action {
                    // TODO: domain type, rename
                    Some(StateFragment {
                        note_commitment: output.body.note_commitment.0.to_bytes().to_vec().into(),
                        ephemeral_key: output.body.ephemeral_key.0.to_vec().into(),
                        encrypted_note: output.body.encrypted_note.to_vec().into(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if should_quarantine {
            todo!();
        } else {
            self.add_note();

            todo!();
        }

        Ok(())
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        todo!()
    }
}

impl ShieldedPool {
    #[instrument(skip(self))]
    fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        transaction_id: [u8; 32],
    ) -> Result<()> {
        // These notes are public, so we don't need a blinding factor for privacy,
        // but since the note commitments are determined by the note contents, we
        // need to have unique (deterministic) blinding factors for each note, so they
        // cannot collide.
        //
        // Hashing the current NCT root is sufficient, since it will change every time
        // we insert a new note.
        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&self.note_commitment_tree.root2().to_bytes())
                .finalize()
                .as_bytes(),
        );

        let note = Note::from_parts(
            *address.diversifier(),
            *address.transmission_key(),
            value,
            blinding_factor,
        )?;
        let note_commitment = note.commit();

        tracing::debug!(?note_commitment, "minted tokens");

        // Scanning assumes that notes are encrypted, so we need to create
        // note ciphertexts, even if the plaintexts are known.  Use the key
        // "1" to ensure we have contributory behaviour in note encryption.
        let esk = ka::Secret::new_from_field(Fr::one());
        let epk = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);

        // Now record the note:

        // 1. Insert it into the NCT
        self.note_commitment_tree.append(&note_commitment);
        // 2. Record its source in the JMT
        self.overlay.lock().unwrap().put(
            format!("shielded_pool/note_commitment_source/{}", note_commitment).into(),
            transaction_id.to_vec(),
        );
        // 3. Update the token supply tracking.
        *self.supply_updates.entry(value.asset_id).or_insert(0) += value.amount as i64;
        // 4. Finally, record it in the pending compact block.
        self.compact_block.fragments.push(StateFragment {
            note_commitment: note_commitment.0.to_bytes().to_vec().into(),
            ephemeral_key: epk.0.to_vec().into(),
            encrypted_note: encrypted_note.to_vec().into(),
        });

        Ok(())
    }

    #[instrument(skip(self))]
    fn write_block(&mut self) -> Result<()> {
        // Write the CompactBlock:
        self.overlay.put_proto(
            format!("shielded_pool/compact_block/{}", self.compact_block.height).into(),
            std::mem::take(&mut self.compact_block),
        );
        // and the note commitment tree data and anchor:
        self.put_nct_anchor();
        self.put_nct()?;

        Ok(())
    }

    fn put_nct(&mut self) -> Result<()> {
        let nct_data = bincode::serialize(&self.note_commitment_tree)?;
        self.overlay
            .lock()
            .unwrap()
            .put(b"shielded_pool/nct_data".into(), nct_data);
        Ok(())
    }

    /// This is an associated function rather than a method,
    /// so that we can call it in the constructor to get the NCT.
    async fn get_nct(overlay: &Overlay) -> Result<NoteCommitmentTree> {
        if let Some(bytes) = overlay
            .lock()
            .unwrap()
            .get(b"shielded_pool/nct_data".into())
            .await?
        {
            bincode::deserialize(&bytes).map_err(Into::into)
        } else {
            Ok(NoteCommitmentTree::new(0))
        }
    }

    fn put_nct_anchor(&mut self) {
        let nct_anchor = self.note_commitment_tree.root2();
        // TODO: should we save a list of historical anchors?
        // Write the NCT anchor both as a value, so we can look it up,
        self.overlay.lock().unwrap().put(
            b"shielded_pool/current_anchor".into(),
            nct_anchor.to_bytes().to_vec(),
        );
        // and as a key, so we can query for it.
        self.overlay.lock().unwrap().put(
            [
                b"shielded_pool/valid_anchors/".to_vec(),
                nct_anchor.to_bytes().to_vec(),
            ]
            .concat()
            .into(),
            Vec::new(),
        );
    }

    async fn check_claimed_anchor(&self, anchor: &merkle::Root) -> Result<()> {
        if self
            .overlay
            .lock()
            .unwrap()
            .get(
                [
                    b"shielded_pool/valid_anchors/".to_vec(),
                    anchor.to_bytes().to_vec(),
                ]
                .concat()
                .into(),
            )
            .await?
            .is_some()
        {
            Ok(())
        } else {
            Err(anyhow!("provided anchor is not a valid NCT root"))
        }
    }
}
