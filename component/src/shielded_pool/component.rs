use std::collections::BTreeSet;

use crate::{
    governance::View as _,
    stake::{validator, View as _},
    Component, Context,
};
use anyhow::{anyhow, Context as _, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::{Fq, Fr};
use penumbra_chain::{
    genesis,
    quarantined::{self, Slashed},
    sync::{AnnotatedNotePayload, CompactBlock},
    Epoch, KnownAssets, NoteSource, View as _,
};
use penumbra_crypto::{
    asset::{self, Asset, Denom},
    ka, note, Address, IdentityKey, Note, NotePayload, Nullifier, One, Value,
    STAKING_TOKEN_ASSET_ID,
};
use penumbra_storage2::State;
use penumbra_tct as tct;
use penumbra_transaction::{
    action::{swap_claim::List as SwapClaimBodyList, Undelegate},
    Action, Transaction,
};
use tendermint::abci;
use tracing::instrument;

use crate::shielded_pool::{consensus_rules, event, state_key, CommissionAmounts};

use super::Delible;

pub struct ShieldedPool {
    note_commitment_tree: tct::Tree,
    /// The in-progress CompactBlock representation of the ShieldedPool changes
    compact_block: CompactBlock,
}

impl ShieldedPool {
    #[instrument(name = "shielded_pool", skip())]
    pub async fn new() -> Self {
        // The NCT is stored outside of the main state,
        // so that the backing format for the NCT isn't consensus-critical.
        // (The NCT data is already committed to by the NCT root, which is in the state).
        let nct = if let Some(tct_bytes) = state.get_nonconsensus("tct")? {
            bincode::deserialize(&tct_bytes)?
        } else {
            penumbra_tct::Tree::new()
        };

        Self {
            note_commitment_tree: nct,
            compact_block: CompactBlock::default(),
        }
    }

    /// Get the current note commitment tree (this may not yet have been committed to the underlying
    /// storage).
    pub fn note_commitment_tree(&self) -> &tct::Tree {
        &self.note_commitment_tree
    }
}

#[async_trait]
impl Component for ShieldedPool {
    #[instrument(name = "shielded_pool", skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            assert_ne!(
                allocation.amount, 0u64,
                "Genesis allocations contain empty note",
            );

            let unit = asset::REGISTRY.parse_unit(&allocation.denom);

            self.state.register_denom(&unit.base()).await.unwrap();
            self.mint_note(
                Value {
                    amount: (allocation.amount * 10u64.pow(unit.exponent().into())).into(),
                    asset_id: unit.id(),
                },
                &allocation.address,
                NoteSource::Genesis,
            )
            .await
            .unwrap();
        }

        // Add current FMD parameters to the initial block.
        self.compact_block.fmd_parameters =
            Some(self.state.get_current_fmd_parameters().await.unwrap());

        // Close the genesis block
        self.finish_nct_block().await;

        // Hard-coded to zero because we are in the genesis block
        self.compact_block.height = 0;

        self.write_compactblock_and_nct().await.unwrap();
    }

    #[instrument(name = "shielded_pool", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "shielded_pool", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        let auth_hash = tx.transaction_body().auth_hash();

        // 1. Check binding signature.
        tx.binding_verification_key()
            .verify(auth_hash.as_ref(), tx.binding_sig())
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
                            output.body.balance_commitment,
                            output.body.note_payload.note_commitment,
                            output.body.note_payload.ephemeral_key,
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
                        .verify(auth_hash.as_ref(), &spend.auth_sig)
                        .context("spend auth signature failed to verify")?;

                    spend
                        .proof
                        .verify(
                            tx.anchor,
                            spend.body.balance_commitment,
                            spend.body.nullifier,
                            spend.body.rk,
                        )
                        .context("a spend proof did not verify")?;

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(spend.body.nullifier);
                }
                // other actions are handled by other components.
                _ => {}
            }
        }

        consensus_rules::stateless::num_clues_equal_to_num_outputs(tx)?;
        consensus_rules::stateless::check_memo_exists_if_outputs_absent_if_not(tx)?;

        Ok(())
    }

    #[instrument(name = "shielded_pool", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        // TODO: rename transaction_body.merkle_root now that we have 2 merkle trees
        self.state.check_claimed_anchor(tx.anchor).await?;

        for spent_nullifier in tx.spent_nullifiers() {
            self.state.check_nullifier_unspent(spent_nullifier).await?;
        }

        // TODO: handle quarantine

        let previous_fmd_parameters = self
            .state
            .get_previous_fmd_parameters()
            .await
            .expect("chain params request must succeed");
        let current_fmd_parameters = self
            .state
            .get_current_fmd_parameters()
            .await
            .expect("chain params request must succeed");
        let height = self.state.get_block_height().await?;
        consensus_rules::stateful::fmd_precision_within_grace_period(
            tx,
            previous_fmd_parameters,
            current_fmd_parameters,
            height,
        )?;

        Ok(())
    }

    #[instrument(name = "shielded_pool", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
        let source = NoteSource::Transaction { id: tx.id() };

        if let Some((epoch, identity_key)) = self.should_quarantine(tx).await {
            for quarantined_output in tx.note_payloads().cloned() {
                // Queue up scheduling this note to be unquarantined: the actual state-writing for
                // all quarantined notes happens during end_block, to avoid state churn
                self.schedule_note(epoch, identity_key, quarantined_output, source)
                    .await;
            }
            for quarantined_spent_nullifier in tx.spent_nullifiers() {
                self.quarantined_spend_nullifier(
                    epoch,
                    identity_key,
                    quarantined_spent_nullifier,
                    source,
                )
                .await;
                ctx.record(event::quarantine_spend(quarantined_spent_nullifier));
            }
        } else {
            for payload in tx.note_payloads().cloned() {
                self.add_note(AnnotatedNotePayload { payload, source })
                    .await;
            }
            for spent_nullifier in tx.spent_nullifiers() {
                self.spend_nullifier(spent_nullifier, source).await;
                ctx.record(event::spend(spent_nullifier));
            }
        }

        // If there was any proposal submitted in the block, ensure we track this so that clients
        // can retain state needed to vote as delegators
        if tx.proposal_submits().next().is_some() {
            self.compact_block.proposal_started = true;
        }
    }

    #[instrument(name = "shielded_pool", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {
        // Get the current block height
        let height = self.height().await;

        // Set the height of the compact block
        self.compact_block.height = height;

        // Handle any pending reward notes from the Staking component
        let notes = self
            .state
            .commission_amounts(height)
            .await
            .unwrap()
            .unwrap_or_default();

        // TODO: should we calculate this here or include it directly within the PendingRewardNote
        // to prevent a potential mismatch between Staking and ShieldedPool?
        let source = NoteSource::FundingStreamReward {
            epoch_index: Epoch::from_height(height, self.state.get_epoch_duration().await.unwrap())
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

        // TODO: execute any scheduled DAO spend transactions for this block

        // Include all output notes from DEX swaps for this block
        self.output_dex_swaps().await;

        // Schedule all unquarantining that was set up in this block
        self.schedule_unquarantine().await;

        // Handle any slashing that occurred in this block, unscheduling all affected notes and
        // nullifiers from future unbonding
        self.process_slashing().await;

        // Process all unquarantining scheduled for this block
        self.process_unquarantine().await;

        // Refund any proposals from this block which are pending refund
        self.process_proposal_refunds().await;

        // Close the block in the NCT
        self.finish_nct_block().await;

        self.write_compactblock_and_nct().await.unwrap();
    }
}

impl ShieldedPool {
    #[instrument(
        skip(self, value, address, source),
        fields(
            position = u64::from(self
                .note_commitment_tree
                .position()
                .unwrap_or_else(|| u64::MAX.into())),
        )
    )]
    async fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        source: NoteSource,
    ) -> Result<()> {
        tracing::debug!(?value, ?address, "minting tokens");
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
        let position: u64 = self
            .note_commitment_tree
            .position()
            .expect("note commitment tree is not full")
            .into();

        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&position.to_le_bytes())
                .finalize()
                .as_bytes(),
        );

        let note = Note::from_parts(*address, value, blinding_factor)?;
        let note_commitment = note.commit();

        // Scanning assumes that notes are encrypted, so we need to create
        // note ciphertexts, even if the plaintexts are known.  Use the key
        // "1" to ensure we have contributory behaviour in note encryption.
        let esk = ka::Secret::new_from_field(Fr::one());
        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);

        // Now record the note and update the total supply:
        self.state
            .update_token_supply(&value.asset_id, i64::from(value.amount))
            .await?;
        self.add_note(AnnotatedNotePayload {
            payload: NotePayload {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            source,
        })
        .await;

        Ok(())
    }

    #[instrument(skip(self, source, payload), fields(note_commitment = ?payload.note_commitment))]
    async fn add_note(&mut self, AnnotatedNotePayload { payload, source }: AnnotatedNotePayload) {
        tracing::debug!("adding note");

        // 1. Insert it into the NCT
        self.note_commitment_tree
            .insert(tct::Witness::Forget, payload.note_commitment)
            .expect("inserting into the note commitment tree never fails");

        // 2. Record its source in the JMT
        self.state
            .set_note_source(payload.note_commitment, source)
            .await;

        // 3. Finally, record it in the pending compact block.
        self.compact_block
            .note_payloads
            .push(AnnotatedNotePayload { payload, source });
    }

    #[instrument(skip(self, source, payload), fields(note_commitment = ?payload.note_commitment))]
    async fn schedule_note(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        payload: NotePayload,
        source: NoteSource,
    ) {
        tracing::debug!("scheduling note");

        // 1. Record its source in the JMT
        self.state
            .set_note_source(payload.note_commitment, source)
            .await;

        // 2. Schedule it in the compact block
        self.compact_block.quarantined.schedule_note(
            epoch,
            identity_key,
            AnnotatedNotePayload { payload, source },
        );
    }

    #[instrument(skip(self, source))]
    async fn spend_nullifier(&mut self, nullifier: Nullifier, source: NoteSource) {
        tracing::debug!("marking as spent");

        // We need to record the nullifier as spent in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so that clients
        // can learn that their note was spent).
        self.state
            .put_domain(
                state_key::spent_nullifier_lookup(nullifier).into(),
                // We don't use the value for validity checks, but writing the source
                // here lets us find out what transaction spent the nullifier.
                source,
            )
            .await;

        self.compact_block.nullifiers.push(nullifier);
    }

    #[instrument(skip(self, source))]
    async fn quarantined_spend_nullifier(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        nullifier: Nullifier,
        source: NoteSource,
    ) {
        // We need to record the nullifier as spent under quarantine in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so clients can learn their note
        // was provisionally spent, pending quarantine period).
        tracing::debug!("marking as spent (currently quarantined)");
        self.state
            .put_domain(
                state_key::quarantined_spent_nullifier_lookup(nullifier).into(),
                // We don't use the value for validity checks, but writing the source
                // here lets us find out what transaction spent the nullifier.
                Delible::Present(source),
            )
            .await;
        // Queue up scheduling this nullifier to be unquarantined: the actual state-writing
        // for all quarantined nullifiers happens during end_block, to avoid state churn
        self.compact_block
            .quarantined
            .schedule_nullifier(epoch, identity_key, nullifier);
    }

    #[instrument(skip(self))]
    async fn write_compactblock_and_nct(&mut self) -> Result<()> {
        // Extract the compact block, resetting it
        let compact_block = std::mem::take(&mut self.compact_block);
        let height = self.height().await;

        // Write the note commitment tree anchor:
        self.state
            .set_nct_anchor(height, self.note_commitment_tree.root())
            .await;
        // Write the current block anchor:
        self.state
            .set_nct_block_anchor(height, compact_block.block_root)
            .await;
        // Write the current epoch anchor, if on an epoch boundary:
        if let Some(epoch_root) = compact_block.epoch_root {
            let epoch_duration = self.epoch_duration().await;
            let index = Epoch::from_height(height, epoch_duration).index;
            self.state.set_nct_epoch_anchor(index, epoch_root).await;
        }

        // Write the CompactBlock:
        self.state.set_compact_block(compact_block).await;

        Ok(())
    }

    /// Finish the block in the NCT.
    #[instrument(skip(self))]
    async fn finish_nct_block(&mut self) {
        // Get the current block height
        let height = self.height().await;

        // Close the block in the TCT
        let block_root = self
            .note_commitment_tree
            .end_block()
            .expect("ending a block in the note commitment tree can never fail");

        // Put the block root in the compact block
        self.compact_block.block_root = block_root;

        // If the block ends an epoch, also close the epoch in the TCT
        if Epoch::from_height(
            height,
            self.state
                .get_chain_params()
                .await
                .expect("chain params request must succeed")
                .epoch_duration,
        )
        .is_epoch_end(height)
        {
            tracing::debug!(?height, "end of epoch");

            // TODO: Put updated FMD parameters in the compact block

            let epoch_root = self
                .note_commitment_tree
                .end_epoch()
                .expect("ending an epoch in the note commitment tree can never fail");

            // Put the epoch root in the compact block
            self.compact_block.epoch_root = Some(epoch_root);
        }
    }

    /// Get the current block height.
    async fn height(&self) -> u64 {
        self.state
            .get_block_height()
            .await
            .expect("block height must be set")
    }

    /// Get the current epoch.
    async fn epoch(&self) -> Epoch {
        // Get the height
        let height = self.height().await;

        // Get the epoch duration
        let epoch_duration = self.epoch_duration().await;

        // The current epoch
        Epoch::from_height(height, epoch_duration)
    }

    /// Get the epoch duration.
    async fn epoch_duration(&self) -> u64 {
        self.state
            .get_epoch_duration()
            .await
            .expect("can get epoch duration")
    }

    /// Returns the epoch and identity key for quarantining a transaction, if it should be
    /// quarantined, otherwise `None`.
    async fn should_quarantine(&self, transaction: &Transaction) -> Option<(u64, IdentityKey)> {
        let validator_identity =
            transaction
                .transaction_body
                .actions
                .iter()
                .find_map(|action| {
                    if let Action::Undelegate(Undelegate {
                        validator_identity, ..
                    }) = action
                    {
                        Some(validator_identity)
                    } else {
                        None
                    }
                })?;

        let validator_bonding_state = self
            .state
            .validator_bonding_state(validator_identity)
            .await
            .expect("validator lookup in state succeeds")
            .expect("validator is present in state");

        let should_quarantine = match validator_bonding_state {
            validator::BondingState::Unbonded => None,
            validator::BondingState::Unbonding { unbonding_epoch } => {
                Some((unbonding_epoch, *validator_identity))
            }
            validator::BondingState::Bonded => {
                let unbonding_epochs = self
                    .state
                    .get_chain_params()
                    .await
                    .expect("can get chain params")
                    .unbonding_epochs;
                Some((
                    self.epoch().await.index + unbonding_epochs,
                    *validator_identity,
                ))
            }
        };

        tracing::debug!(?should_quarantine, "should quarantine");

        should_quarantine
    }

    async fn schedule_unquarantine(&mut self) {
        // First, we group all the scheduled quarantined notes by unquarantine epoch, in the process
        // resetting the quarantine field of the component
        for (&unbonding_epoch, scheduled) in self.compact_block.quarantined.iter() {
            self.state
                .schedule_unquarantine(unbonding_epoch, scheduled.clone())
                .await
                .expect("scheduling unquarantine must succeed");
        }
    }

    // Process any slashing that occrred in this block.
    async fn process_slashing(&mut self) {
        self.compact_block.slashed.extend(
            self.state
                .unschedule_all_slashed()
                .await
                .expect("can unschedule slashed"),
        );
    }

    // Process any notes/nullifiers due to be unquarantined in this block, if it's an
    // epoch-ending block
    #[instrument(skip(self))]
    async fn process_unquarantine(&mut self) {
        let this_epoch = self.epoch().await;

        if this_epoch.is_epoch_end(self.height().await) {
            for (_, per_validator) in self
                .state
                .scheduled_to_apply(this_epoch.index)
                .await
                .expect("can look up quarantined for this epoch")
            {
                // For all the note payloads scheduled for unquarantine now, remove them from
                // quarantine and add them to the proper notes for this block
                for note_payload in per_validator.note_payloads {
                    tracing::debug!(?note_payload, "unquarantining note");
                    self.add_note(note_payload).await;
                }
                // For all the nullifiers scheduled for unquarantine now, remove them from
                // quarantine and add them to the proper nullifiers for this block
                for nullifier in per_validator.nullifiers {
                    let note_source = self
                        .state
                        .unquarantine_nullifier(nullifier)
                        .await
                        .expect("can try to unquarantine nullifier")
                        .expect("nullifier to unquarantine has source");
                    tracing::debug!(?nullifier, "unquarantining nullifier");
                    self.spend_nullifier(nullifier, note_source).await;
                }
            }
        }
    }

    #[instrument(skip(self))]
    async fn process_proposal_refunds(&mut self) {
        let block_height = self.height().await;

        for (proposal_id, address, value) in self
            .state
            .proposal_refunds(block_height)
            .await
            .expect("proposal refunds can be fetched")
        {
            self.mint_note(
                value,
                &address,
                NoteSource::ProposalDepositRefund { proposal_id },
            )
            .await
            .expect("can mint proposal deposit refund");
        }
    }

    #[instrument(skip(self))]
    async fn output_dex_swaps(&mut self) {
        let block_height = self.height().await;

        for claimed_swap in self
            .state
            .claimed_swap_outputs(block_height)
            .await
            .expect("claimed swap outputs can be fetched")
            .expect("claimed swap outputs was set")
            .0
        {
            let (swap_claim, txid) = (claimed_swap.0, claimed_swap.1);
            let source = NoteSource::Transaction { id: txid };
            let payload_1 = swap_claim.output_1;
            let payload_2 = swap_claim.output_2;
            self.add_note(AnnotatedNotePayload {
                payload: payload_1,
                source,
            })
            .await;
            self.add_note(AnnotatedNotePayload {
                payload: payload_2,
                source,
            })
            .await;

            // Also spend the nullifier.
            self.spend_nullifier(swap_claim.nullifier, source).await;
        }
    }
}

/// Extension trait providing read/write access to shielded pool data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: StateExt {
    async fn token_supply(&self, asset_id: &asset::Id) -> Result<Option<u64>> {
        self.get_proto(state_key::token_supply(asset_id).into())
            .await
    }

    #[instrument(skip(self, change))]
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
                .checked_sub(change.unsigned_abs())
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
        tracing::debug!(?current_supply, ?new_supply, ?change);

        self.put_proto(key, new_supply).await;
        Ok(())
    }

    async fn known_assets(&self) -> Result<KnownAssets> {
        Ok(self
            .get_domain(state_key::known_assets().into())
            .await?
            .unwrap_or_default())
    }

    async fn denom_by_asset(&self, asset_id: &asset::Id) -> Result<Option<Denom>> {
        self.get_domain(state_key::denom_by_asset(asset_id).into())
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
            self.put_domain(state_key::denom_by_asset(&id).into(), denom.clone())
                .await;
            // ... and we want to record it in the list of known asset IDs
            // (this requires reading the whole list, which is sad, but hopefully
            // we don't do this often).
            let mut known_assets = self.known_assets().await?;
            known_assets.0.push(Asset {
                id,
                denom: denom.clone(),
            });
            self.put_domain(state_key::known_assets().into(), known_assets)
                .await;
            Ok(())
        }
    }

    async fn set_note_source(&self, note_commitment: note::Commitment, source: NoteSource) {
        self.put_domain(
            state_key::note_source(note_commitment).into(),
            Delible::Present(source),
        )
        .await
    }

    // Returns whether the note was presently quarantined.
    async fn roll_back_note(&self, commitment: note::Commitment) -> Result<Option<NoteSource>> {
        // Get the note source of the note (or empty vec if already applied or rolled back)
        let source = self
            .get_domain::<Delible<NoteSource>, _>(state_key::note_source(commitment).into())
            .await?
            .expect("can't roll back note that was never created")
            .into();

        // Delete the note from the set of all notes
        self.put_domain(state_key::note_source(commitment).into(), Delible::Deleted)
            .await;

        Ok(source)
    }

    async fn note_source(&self, note_commitment: note::Commitment) -> Result<Option<NoteSource>> {
        Ok(self
            .get_domain::<Delible<NoteSource>, _>(state_key::note_source(note_commitment).into())
            .await?
            .unwrap_or_default()
            .into())
    }

    async fn set_compact_block(&self, compact_block: CompactBlock) {
        let height = compact_block.height;
        self.put_domain(state_key::compact_block(height).into(), compact_block)
            .await
    }

    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        self.get_domain(state_key::compact_block(height).into())
            .await
    }

    async fn set_nct_anchor(&self, height: u64, nct_anchor: tct::Root) {
        tracing::debug!(?height, ?nct_anchor, "writing anchor");

        // Write the NCT anchor both as a value, so we can look it up,
        self.put_domain(state_key::anchor_by_height(height).into(), nct_anchor)
            .await;
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::anchor_lookup(nct_anchor).into(),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height,
        )
        .await;
    }

    async fn set_nct_block_anchor(&self, height: u64, nct_block_anchor: tct::builder::block::Root) {
        tracing::debug!(?height, ?nct_block_anchor, "writing block anchor");

        // Write the NCT block anchor both as a value, so we can look it up,
        self.put_domain(
            state_key::block_anchor_by_height(height).into(),
            nct_block_anchor,
        )
        .await;
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::block_anchor_lookup(nct_block_anchor).into(),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height,
        )
        .await;
    }

    async fn set_nct_epoch_anchor(&self, index: u64, nct_block_anchor: tct::builder::epoch::Root) {
        tracing::debug!(?index, ?nct_block_anchor, "writing epoch anchor");

        // Write the NCT epoch anchor both as a value, so we can look it up,
        self.put_domain(
            state_key::epoch_anchor_by_index(index).into(),
            nct_block_anchor,
        )
        .await;
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::epoch_anchor_lookup(nct_block_anchor).into(),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            index,
        )
        .await;
    }

    /// Checks whether a claimed NCT anchor is a previous valid state root.
    async fn check_claimed_anchor(&self, anchor: tct::Root) -> Result<()> {
        if let Some(anchor_height) = self
            .get_proto::<u64>(state_key::anchor_lookup(anchor).into())
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

    // Returns the source if the nullifier was in quarantine already
    #[instrument(skip(self))]
    async fn unquarantine_nullifier(&self, nullifier: Nullifier) -> Result<Option<NoteSource>> {
        tracing::debug!("removing quarantined nullifier");

        // Get the note source of the nullifier (or empty vec if already applied or rolled back)
        let source = self
            .get_domain::<Delible<NoteSource>, _>(
                state_key::quarantined_spent_nullifier_lookup(nullifier).into(),
            )
            .await?
            .expect("can't unquarantine nullifier that was never quarantined")
            .into();

        // Delete the nullifier from the quarantine set
        self.put_domain(
            state_key::quarantined_spent_nullifier_lookup(nullifier).into(),
            Delible::Deleted,
        )
        .await;

        Ok(source)
    }

    #[instrument(skip(self))]
    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(source) = self
            .get_domain::<NoteSource, _>(state_key::spent_nullifier_lookup(nullifier).into())
            .await?
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                source,
            ));
        }

        if let Some(source) = self
            .get_domain::<Delible<NoteSource>, _>(
                state_key::quarantined_spent_nullifier_lookup(nullifier).into(),
            )
            .await?
            .and_then(<Option<NoteSource>>::from)
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?} (currently quarantined)",
                nullifier,
                source,
            ));
        }

        Ok(())
    }

    async fn scheduled_to_apply(&self, epoch: u64) -> Result<quarantined::Scheduled> {
        Ok(self
            .get_domain(state_key::scheduled_to_apply(epoch).into())
            .await?
            .unwrap_or_default())
    }

    async fn schedule_unquarantine(
        &self,
        epoch: u64,
        scheduled: quarantined::Scheduled,
    ) -> Result<()> {
        let mut updated_quarantined = self.scheduled_to_apply(epoch).await?;
        updated_quarantined.extend(scheduled);
        self.put_domain(
            state_key::scheduled_to_apply(epoch).into(),
            updated_quarantined,
        )
        .await;
        Ok(())
    }

    // Unschedule the unquarantining of all notes and nullifiers for the given validator, in any
    // epoch which could possibly still be unbonding
    async fn unschedule_all_slashed(&self) -> Result<Vec<IdentityKey>> {
        let height = self.get_block_height().await?;
        let epoch_duration = self.get_epoch_duration().await?;
        let this_epoch = Epoch::from_height(height, epoch_duration);
        let unbonding_epochs = self.get_chain_params().await?.unbonding_epochs;

        let slashed: Slashed = self
            .get_domain(state_key::slashed_validators(height).into())
            .await?
            .unwrap_or_default();

        for epoch in this_epoch.index.saturating_sub(unbonding_epochs)..=this_epoch.index {
            let mut updated_scheduled = self.scheduled_to_apply(epoch).await?;
            for &identity_key in &slashed.validators {
                let unbonding = updated_scheduled.unschedule_validator(identity_key);
                // Now we also ought to remove these nullifiers and notes from quarantine without
                // applying them:
                for &nullifier in unbonding.nullifiers.iter() {
                    self.unquarantine_nullifier(nullifier).await?;
                }
                for note_payload in unbonding.note_payloads.iter() {
                    self.roll_back_note(note_payload.payload.note_commitment)
                        .await?;
                }
            }
            // We're removed all the scheduled notes and nullifiers for this epoch and identity key:
            self.put_domain(
                state_key::scheduled_to_apply(epoch).into(),
                updated_scheduled,
            )
            .await;
        }

        Ok(slashed.validators)
    }

    // TODO: rename to something more generic ("minted notes"?) that can
    // be used with IBC transfers, and fix up the path and proto

    async fn commission_amounts(&self, height: u64) -> Result<Option<CommissionAmounts>> {
        self.get_domain(state_key::commission_amounts(height).into())
            .await
    }

    async fn set_commission_amounts(&self, height: u64, notes: CommissionAmounts) {
        self.put_domain(state_key::commission_amounts(height).into(), notes)
            .await
    }

    async fn claimed_swap_outputs(&self, height: u64) -> Result<Option<SwapClaimBodyList>> {
        self.get_domain(state_key::claimed_swap_outputs(height).into())
            .await
    }

    async fn set_claimed_swap_outputs(&self, height: u64, claims: SwapClaimBodyList) {
        self.put_domain(state_key::claimed_swap_outputs(height).into(), claims)
            .await
    }
}

impl<T: StateExt> View for T {}
