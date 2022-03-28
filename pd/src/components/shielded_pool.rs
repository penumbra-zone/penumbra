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
    Note, One, Value,
};
use penumbra_proto::light_wallet::{CompactBlock, StateFragment};
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, Overlay};
use crate::{genesis, WriteOverlayExt};

// Stub component
pub struct ShieldedPool {
    overlay: Overlay,
    note_commitment_tree: NoteCommitmentTree,
    supply_updates: BTreeMap<asset::Id, i64>,
}

#[async_trait]
impl Component for ShieldedPool {
    async fn new(overlay: Overlay) -> Result<Self> {
        // TODO: Component::new() needs to be async
        let note_commitment_tree = NoteCommitmentTree::new(0);
        Ok(Self {
            overlay,
            note_commitment_tree,
            supply_updates: Default::default(),
        })
    }

    fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        let mut fragments = Vec::new();
        let mut genesis_supply = BTreeMap::new();

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

            // These notes are public, so we don't need a blinding factor for privacy,
            // but since the note commitments are determined by the note contents, we
            // want to have unique (deterministic) blinding factors for each one.
            let blinding_factor = blake2b_simd::Params::default()
                .personal(b"genesis_note")
                .to_state()
                .update(app_state.chain_params.chain_id.as_bytes())
                .update(&(fragments.len() as u64).to_le_bytes())
                .finalize();

            let note = Note::from_parts(
                *allocation.address.diversifier(),
                *allocation.address.transmission_key(),
                Value {
                    amount: allocation.amount,
                    asset_id: base_denom.id(),
                },
                Fq::from_le_bytes_mod_order(blinding_factor.as_bytes()),
            )?;
            let note_commitment = note.commit();

            // Scanning assumes that notes are encrypted, so we need to create
            // note ciphertexts, even if the plaintexts are known.  Use the key
            // "1" to ensure we have contributory behaviour in note encryption.
            let esk = ka::Secret::new_from_field(Fr::one());
            let epk = esk.diversified_public(&note.diversified_generator());
            let encrypted_note = note.encrypt(&esk);

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

            // Now record the note:

            // 1. Insert it into the NCT
            self.note_commitment_tree.append(&note_commitment);
            // 2. Record its source in the JMT
            self.overlay.lock().unwrap().put(
                format!("shielded_pool/note_commitment_source/{}", note_commitment).into(),
                transaction_id,
            );
            // 3. Queue it for inclusion into the genesis CompactBlock
            fragments.push(StateFragment {
                note_commitment: note_commitment.0.to_bytes().to_vec().into(),
                ephemeral_key: epk.0.to_vec().into(),
                encrypted_note: encrypted_note.to_vec().into(),
            });
            // 4. Update the genesis token supply tracking.
            genesis_supply
                .entry(base_denom.id())
                .or_insert((base_denom, 0))
                .1 += allocation.amount;
        }

        // Finally, write the genesis CompactBlock:
        let compact_block = CompactBlock {
            height: 0,
            nullifiers: Vec::new(),
            fragments,
        };
        self.overlay.put_proto(
            format!("shielded_pool/compact_block/{}", compact_block.height).into(),
            compact_block,
        );
        // and the note commitment tree data and anchor:
        let nct_anchor = self.note_commitment_tree.root2();
        let nct_data = bincode::serialize(&self.note_commitment_tree)?;
        self.overlay.lock().unwrap().put(
            b"shielded_pool/nct_anchor".into(),
            nct_anchor.to_bytes().to_vec(),
        );
        self.overlay
            .lock()
            .unwrap()
            .put(b"shielded_pool/nct_data".into(), nct_data);

        Ok(())
    }

    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        todo!()
    }

    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        todo!()
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn execute_tx(&mut self, _tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        todo!()
    }
}

impl ShieldedPool {
    #[tracing::instrument(skip(self))]
    fn mint_tokens(&mut self, value: Value, transaction_id: [u8; 32]) -> StateFragment {
        // These notes are public, so we don't need a blinding factor for privacy,
        // but since the note commitments are determined by the note contents, we
        // need to have unique (deterministic) blinding factors for each note, so they
        // cannot collide.
        //
        // Hashing the current NCT root is sufficient, since it will change every time
        // we insert a new note.
        let blinding_factor = blake2b_simd::Params::default()
            .personal(b"PenumbraMint")
            .to_state()
            .update(&self.note_commitment_tree.root2().to_bytes())
            .finalize();

        todo!()
    }
}
