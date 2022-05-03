use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::{Fq, Fr};
use penumbra_chain::{genesis, sync::CompactBlock, Epoch, KnownAssets, NoteSource, View as _};
use penumbra_component::Component;
use penumbra_crypto::{
    asset::{self, Asset, Denom},
    ka,
    merkle::{self, Frontier, NoteCommitmentTree, TreeExt},
    note, Address, Note, Nullifier, One, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::{action::output, Action, Transaction};
use tendermint::abci;
use tracing::instrument;

use crate::CommissionAmounts;

// Stub component
pub struct ShieldedPool {
    state: State,
    note_commitment_tree: NoteCommitmentTree,
    /// The in-progress CompactBlock representation of the ShieldedPool changes
    compact_block: CompactBlock,
}

#[async_trait]
impl Component for ShieldedPool {
    #[instrument(name = "shielded_pool", skip(state))]
    async fn new(state: State) -> Self {
        let note_commitment_tree = Self::get_nct(&state).await.unwrap();

        Self {
            state,
            note_commitment_tree,
            compact_block: Default::default(),
        }
    }

    #[instrument(name = "shielded_pool", skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            assert_ne!(
                allocation.amount, 0,
                "Genesis allocations contain empty note",
            );

            let denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .ok_or_else(|| {
                    anyhow!(
                        "Genesis denomination {} is not a base denom",
                        allocation.denom
                    )
                })
                .unwrap();

            self.state.register_denom(&denom).await.unwrap();
            self.mint_note(
                Value {
                    amount: allocation.amount,
                    asset_id: denom.id(),
                },
                &allocation.address,
                NoteSource::Genesis,
            )
            .await
            .unwrap();
        }

        self.compact_block.height = 0;
        self.write_compactblock_and_nct().await.unwrap();
    }

    #[instrument(name = "shielded_pool", skip(self, _begin_block))]
    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "shielded_pool", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        let sighash = tx.transaction_body().sighash();

        // 1. Check binding signature.
        tx.binding_verification_key()
            .verify(&sighash, tx.binding_sig())
            .context("binding signature failed to verify")?;

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();

        for action in tx.transaction_body().actions {
            match action {
                Action::Output(output) => {
                    if output
                        .proof
                        .verify(
                            output.value_commitment,
                            output.body.note_commitment,
                            output.body.ephemeral_key,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("An output proof did not verify"));
                    }
                }
                Action::Spend(spend) => {
                    spend
                        .body
                        .rk
                        .verify(&sighash, &spend.auth_sig)
                        .context("spend auth signature failed to verify")?;

                    if spend
                        .body
                        .proof
                        .verify(
                            tx.transaction_body().merkle_root,
                            spend.body.value_commitment,
                            spend.body.nullifier.clone(),
                            spend.body.rk,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("A spend proof did not verify"));
                    }

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(spend.body.nullifier.clone());
                }
                Action::Delegate(_delegate) => {
                    // Handled in the `Staking` component.
                }
                Action::Undelegate(_undelegate) => {
                    // Handled in the `Staking` component.
                }
                Action::ValidatorDefinition(_validator) => {
                    // Handled in the `Staking` component.
                }
                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow::anyhow!("unsupported action"));
                }
            }
        }

        Ok(())
    }

    #[instrument(name = "shielded_pool", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        // TODO: rename transaction_body.merkle_root now that we have 2 merkle trees
        self.state
            .check_claimed_anchor(&tx.transaction_body.merkle_root)
            .await?;

        for spent_nullifier in tx.spent_nullifiers() {
            self.state.check_nullifier_unspent(spent_nullifier).await?;
        }

        // TODO: handle quarantine
        Ok(())
    }

    #[instrument(name = "shielded_pool", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        let _should_quarantine = tx
            .transaction_body
            .actions
            .iter()
            .any(|action| matches!(action, Action::Undelegate { .. }));

        let source = NoteSource::Transaction { id: tx.id() };

        /*
        if should_quarantine {
            tracing::warn!("skipping processing, TODO: implement");
        } else {
         */
        for compact_output in tx.output_bodies() {
            self.add_note(compact_output, source).await;
        }
        for spent_nullifier in tx.spent_nullifiers() {
            // We need to record the nullifier as spent in the JMT (to prevent
            // double spends), as well as in the CompactBlock (so that clients
            // can learn that their note was spent).
            self.state.spend_nullifier(spent_nullifier, source).await;
            self.compact_block.nullifiers.push(spent_nullifier);
        }
        //}
    }

    #[instrument(name = "shielded_pool", skip(self, end_block))]
    async fn end_block(&mut self, end_block: &abci::request::EndBlock) {
        // Set the height of the compact block, now that we got it in end_block
        self.compact_block.height = end_block.height as u64;

        // Handle any pending reward notes from the Staking component
        let notes = self
            .state
            .commission_amounts(self.compact_block.height)
            .await
            .unwrap()
            .unwrap_or_default();

        // TODO: should we calculate this here or include it directly within the PendingRewardNote
        // to prevent a potential mismatch between Staking and ShieldedPool?
        let source = NoteSource::FundingStreamReward {
            epoch_index: Epoch::from_height(
                self.compact_block.height,
                self.state.get_epoch_duration().await.unwrap(),
            )
            .index,
        };

        for note in notes.notes {
            self.mint_note(
                Value {
                    amount: note.amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                &note.destination,
                source,
            )
            .await
            .unwrap();
        }

        self.write_compactblock_and_nct().await.unwrap();
    }
}

impl ShieldedPool {
    #[instrument(skip(self))]
    async fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        source: NoteSource,
    ) -> Result<()> {
        // These notes are public, so we don't need a blinding factor for privacy,
        // but since the note commitments are determined by the note contents, we
        // need to have unique (deterministic) blinding factors for each note, so they
        // cannot collide.
        //
        // Hashing the current NCT root is sufficient, since it will change every time
        // we insert a new note.

        // However, in our current implementation, computing the NCT root is very slow...
        /*
        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&self.note_commitment_tree.root2().to_bytes())
                .finalize()
                .as_bytes(),
        );
        */

        // ... so just hash the current position instead.
        let position = self
            .note_commitment_tree
            .bridges()
            .last()
            .map(|b| b.frontier().position().into())
            // If there are no bridges, the tree is empty
            .unwrap_or(0u64);
        tracing::debug!(
            ?position,
            "using NCT position for uniqueness in minted note"
        );
        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&position.to_le_bytes())
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
        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);

        // Now record the note and update the total supply:
        self.state
            .update_token_supply(&value.asset_id, value.amount as i64)
            .await?;
        self.add_note(
            output::Body {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            source,
        )
        .await;

        Ok(())
    }

    #[instrument(skip(self, source, output_body))]
    async fn add_note(&mut self, output_body: output::Body, source: NoteSource) {
        tracing::debug!(commitment = ?output_body.note_commitment, "appending to NCT in component");
        // 1. Insert it into the NCT
        self.note_commitment_tree
            .append(&output_body.note_commitment);
        // 2. Record its source in the JMT
        self.state
            .set_note_source(&output_body.note_commitment, source)
            .await;
        // 3. Finally, record it in the pending compact block.
        self.compact_block.outputs.push(output_body);
    }

    #[instrument(skip(self))]
    async fn write_compactblock_and_nct(&mut self) -> Result<()> {
        // Write the CompactBlock:
        self.state
            .set_compact_block(std::mem::take(&mut self.compact_block))
            .await;
        // and the note commitment tree data and anchor:
        self.state
            .set_nct_anchor(self.compact_block.height, self.note_commitment_tree.root2())
            .await;
        self.put_nct().await?;

        Ok(())
    }

    /// This is not part of the View trait because the NCT isn't a domain
    /// type, and we'll be replacing it anyways, so there's not much point
    /// implementing one now.  When switching to the TCT we should revisit.
    async fn put_nct(&mut self) -> Result<()> {
        let nct_data = bincode::serialize(&self.note_commitment_tree)?;
        self.state
            .write()
            .await
            .put(b"shielded_pool/nct_data".into(), nct_data);
        Ok(())
    }

    /// This is an associated function rather than a method,
    /// so that we can call it in the constructor to get the NCT.
    /// NOTE: we may not need that any more now that we can use an
    /// State on an empty database.
    async fn get_nct(state: &State) -> Result<NoteCommitmentTree> {
        if let Ok(Some(bytes)) = state
            .read()
            .await
            .get(b"shielded_pool/nct_data".into())
            .await
        {
            bincode::deserialize(&bytes).map_err(Into::into)
        } else {
            Ok(NoteCommitmentTree::new(0))
        }
    }
}

/// Extension trait providing read/write access to shielded pool data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: StateExt {
    async fn token_supply(&self, asset_id: &asset::Id) -> Result<Option<u64>> {
        self.get_proto(format!("shielded_pool/assets/{}/token_supply", asset_id).into())
            .await
    }

    #[instrument(skip(self))]
    async fn update_token_supply(&self, asset_id: &asset::Id, change: i64) -> Result<()> {
        let key = format!("shielded_pool/assets/{}/token_supply", asset_id).into();
        let current_supply = match self.get_proto(key).await {
            Ok(Some(value)) => value,
            Ok(None) => 0u64,
            // We want to handle the MissingRootError specially here, so that we can
            // use 0 as a default value if the tree is empty, without ignoring all errors.
            Err(e) if e.downcast_ref::<jmt::MissingRootError>().is_some() => 0u64,
            Err(e) => return Err(e),
        };

        // TODO: replace with a single checked_add_signed call when mixed_integer_ops lands in stable
        let new_supply = if change < 0 {
            current_supply
                .checked_sub(change.abs() as u64)
                .ok_or_else(|| {
                    anyhow!(
                        "underflow updating token supply {} with delta {}",
                        current_supply,
                        change
                    )
                })?
        } else {
            current_supply.checked_add(change as u64).ok_or_else(|| {
                anyhow!(
                    "overflow updating token supply {} with delta {}",
                    current_supply,
                    change
                )
            })?
        };
        tracing::debug!(?current_supply, ?new_supply);

        self.put_proto(key, new_supply).await;
        Ok(())
    }

    async fn known_assets(&self) -> Result<KnownAssets> {
        Ok(self
            .get_domain("shielded_pool/known_assets".into())
            .await?
            .unwrap_or_default())
    }

    async fn denom_by_asset(&self, asset_id: &asset::Id) -> Result<Option<Denom>> {
        self.get_domain(format!("shielded_pool/assets/{}/denom", asset_id).into())
            .await
    }

    #[instrument(skip(self))]
    async fn register_denom(&self, denom: &Denom) -> Result<()> {
        let id = denom.id();
        if self.denom_by_asset(&id).await?.is_some() {
            tracing::debug!(?denom, ?id, "skipping existing denom");
            Ok(())
        } else {
            tracing::debug!(?denom, ?id, "registering new denom");
            // We want to be able to query for the denom by asset ID...
            self.put_domain(
                format!("shielded_pool/assets/{}/denom", id).into(),
                denom.clone(),
            )
            .await;
            // ... and we want to record it in the list of known asset IDs
            // (this requires reading the whole list, which is sad, but hopefully
            // we don't do this often).
            let mut known_assets = self.known_assets().await?;
            known_assets.0.push(Asset {
                id,
                denom: denom.clone(),
            });
            self.put_domain("shielded_pool/known_assets".into(), known_assets)
                .await;
            Ok(())
        }
    }

    async fn set_note_source(&self, note_commitment: &note::Commitment, source: NoteSource) {
        self.put_domain(
            format!("shielded_pool/note_source/{}", note_commitment).into(),
            source,
        )
        .await
    }

    async fn note_source(&self, note_commitment: &note::Commitment) -> Result<Option<NoteSource>> {
        self.get_domain(format!("shielded_pool/note_source/{}", note_commitment).into())
            .await
    }

    async fn set_compact_block(&self, compact_block: CompactBlock) {
        self.put_domain(
            format!("shielded_pool/compact_block/{}", compact_block.height).into(),
            compact_block,
        )
        .await
    }

    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        self.get_domain(format!("shielded_pool/compact_block/{}", height).into())
            .await
    }

    async fn set_nct_anchor(&self, height: u64, anchor: merkle::Root) {
        tracing::debug!(?height, ?anchor, "writing anchor");

        // Write the NCT anchor both as a value, so we can look it up,
        self.put_domain(
            format!("shielded_pool/nct_anchor/{}", height).into(),
            anchor.clone(),
        )
        .await;
        // and as a key, so we can query for it.
        self.put_proto(
            format!("shielded_pool/valid_anchors/{}", anchor).into(),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height,
        )
        .await;
    }

    /// Checks whether a claimed NCT anchor is a previous valid state root.
    async fn check_claimed_anchor(&self, anchor: &merkle::Root) -> Result<()> {
        if let Some(anchor_height) = self
            .get_proto::<u64>(format!("shielded_pool/valid_anchors/{}", anchor).into())
            .await?
        {
            tracing::debug!(?anchor, ?anchor_height, "anchor is valid");
            Ok(())
        } else {
            Err(anyhow!(
                "provided anchor {} is not a valid NCT root",
                anchor
            ))
        }
    }

    #[instrument(skip(self))]
    async fn spend_nullifier(&self, nullifier: Nullifier, source: NoteSource) {
        self.put_proto(
            format!("shielded_pool/spent_nullifiers/{}", nullifier).into(),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            // TODO: NoteSource proto?
            source.to_bytes().to_vec(),
        )
        .await;
    }

    #[instrument(skip(self))]
    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(source_bytes) = self
            .get_proto::<Vec<u8>>(format!("shielded_pool/spent_nullifiers/{}", nullifier).into())
            .await?
        {
            // TODO: NoteSource proto?
            let source_bytes: [u8; 32] = source_bytes.try_into().unwrap();
            let source = NoteSource::try_from(source_bytes).expect("source is validly encoded");
            Err(anyhow!(
                "Nullifier {} was already spent in {:?}",
                nullifier,
                source
            ))
        } else {
            Ok(())
        }
    }

    // TODO: rename to something more generic ("minted notes"?) that can
    // be used with IBC transfers, and fix up the path and proto

    async fn commission_amounts(&self, height: u64) -> Result<Option<CommissionAmounts>> {
        self.get_domain(format!("staking/commission_amounts/{}", height).into())
            .await
    }

    async fn set_commission_amounts(&self, height: u64, notes: CommissionAmounts) {
        self.put_domain(
            format!("staking/commission_amounts/{}", height).into(),
            notes,
        )
        .await
    }
}

impl<T: StateExt> View for T {}
