use std::collections::{BTreeMap, VecDeque};

use anyhow::Result;
use ark_ff::PrimeField;
use decaf377::{Fq, Fr};
use jmt::TreeWriterAsync;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    asset,
    asset::{Denom, Id},
    ka,
    merkle::{self, Frontier, NoteCommitmentTree, TreeExt},
    Note, One, Value,
};
use penumbra_proto::Protobuf;
use penumbra_stake::{FundingStream, RateDataById, ValidatorStateName};
use sqlx::{query, Pool, Postgres};
use tendermint::block;
use tokio::sync::watch;

use super::jellyfish;
use crate::{
    components::validator_set::ValidatorSet,
    genesis,
    pending_block::QuarantineGroup,
    verify::{NoteData, PositionedNoteData},
    PendingBlock, NUM_RECENT_ANCHORS,
};

#[derive(Debug)]
pub struct Writer {
    pub(super) pool: Pool<Postgres>,
    // A state::Reader instance that uses the same connection pool as this
    // Writer, allowing it to read (e.g., for transaction verification) without
    // risking contention with other users of the read connection pool.
    pub(super) private_reader: super::Reader,
    //pub(super) tmp: evmap::WriteHandle<&'static str, String>,
    // Push channels for chain state
    pub(super) chain_params_tx: watch::Sender<ChainParams>,
    pub(super) height_tx: watch::Sender<block::Height>,
    pub(super) next_rate_data_tx: watch::Sender<RateDataById>,
    pub(super) valid_anchors_tx: watch::Sender<VecDeque<merkle::Root>>,
}

impl Writer {
    /// Initializes in-memory caches / notification channels.
    /// Called by `state::new()` on init, and when reloading the state after init_chain
    pub async fn init_caches(&self) -> Result<()> {
        let chain_params = self
            .private_reader
            .genesis_configuration()
            .await?
            .chain_params;
        let height = self.private_reader.height();
        let next_rate_data = self.private_reader.next_rate_data().await?;
        let valid_anchors = self
            .private_reader
            .recent_anchors(NUM_RECENT_ANCHORS)
            .await?;

        // Sends fail if every receiver has been dropped, which is not our problem.
        let _ = self.chain_params_tx.send(chain_params);
        let _ = self.height_tx.send(height);
        let _ = self.next_rate_data_tx.send(next_rate_data);
        let _ = self.valid_anchors_tx.send(valid_anchors);

        Ok(())
    }

    /// Borrow a private `state::Reader` instance that uses the same connection
    /// pool as this writer.  This allows the writer to read data from the
    /// database without contention from other `state::Reader`s.
    pub fn private_reader(&self) -> &super::Reader {
        &self.private_reader
    }

    /// Commits the genesis config to the database, prior to the first block commit.
    ///
    /// The database queries here have quite a bit of overlap with the queries in
    /// commit_block(), but this is because the genesis setup is better treated
    /// as a simple special case rather than creating a fake pseudo-block.
    pub async fn commit_genesis(&self, genesis_config: &genesis::AppState) -> Result<Vec<u8>> {
        let mut dbtx = self.pool.begin().await?;

        let genesis_bytes = serde_json::to_vec(&genesis_config)?;

        // ON CONFLICT is excluded here so that an error is raised
        // if genesis config is attempted to be set more than once
        query!(
            r#"
            INSERT INTO blobs (id, data) VALUES ('gc', $1)
            "#,
            &genesis_bytes[..]
        )
        .execute(&mut dbtx)
        .await?;

        // TODO(future): rewrite all of this as
        // ValidatorSet::commit_genesis(...)
        // ShieldedPool::commit_genesis(...)
        // and figure out how to pass any required state between those methods

        // Delegations require knowing the rates for the next epoch, so
        // pre-populate with 0 reward => exchange rate 1 for the current
        // (index 0) and next (index 1) epochs.
        for epoch in [0, 1] {
            query!(
                "INSERT INTO base_rates (
                epoch,
                base_reward_rate,
                base_exchange_rate
            ) VALUES ($1, $2, $3)",
                epoch,
                0,
                1_0000_0000
            )
            .execute(&mut dbtx)
            .await?;
        }

        for genesis::ValidatorPower { validator, power } in &genesis_config.validators {
            query!(
                "INSERT INTO validators (
                    identity_key,
                    consensus_key,
                    sequence_number,
                    name,
                    website,
                    description,
                    voting_power,
                    validator_state,
                    unbonding_epoch
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                validator.identity_key.encode_to_vec(),
                validator.consensus_key.to_bytes(),
                i64::try_from(validator.sequence_number)?,
                validator.name,
                validator.website,
                validator.description,
                i64::try_from(power.value())?,
                ValidatorStateName::Active.to_str().to_string(),
                Option::<i64>::None,
            )
            .execute(&mut dbtx)
            .await?;

            for FundingStream { address, rate_bps } in validator.funding_streams.as_ref() {
                query!(
                    "INSERT INTO validator_fundingstreams (
                        identity_key,
                        address,
                        rate_bps
                    ) VALUES ($1, $2, $3)",
                    validator.identity_key.encode_to_vec(),
                    address.to_string(),
                    *rate_bps as i32,
                )
                .execute(&mut dbtx)
                .await?;
            }

            // The initial voting power is set from the genesis configuration,
            // but later, it's recomputed based on the size of each validator's
            // delegation pool.  Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current (index 0) and next (index 1) epochs.
            for epoch in [0, 1] {
                query!(
                    "INSERT INTO validator_rates (
                    identity_key,
                    epoch,
                    validator_reward_rate,
                    validator_exchange_rate
                ) VALUES ($1, $2, $3, $4)",
                    validator.identity_key.encode_to_vec(),
                    epoch,
                    0,
                    1_0000_0000i64, // 1 represented as 1e8
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // build a note commitment tree
        let mut note_commitment_tree = NoteCommitmentTree::new(0);

        // iterate over genesis allocations
        //
        // - add the note to the NCT
        // - insert the note into the database as appropriate (#374) https://github.com/penumbra-zone/penumbra/issues/374
        // - accumulate the value into a supply tracker
        //
        // The blinding factor needs to be unique per genesis note
        // so a monotonically increasing reward counter is used.
        let mut reward_counter: u64 = 0;
        let mut supply_updates: BTreeMap<Id, (Denom, u64)> = BTreeMap::new();
        let mut notes = Vec::new();
        for allocation in &genesis_config.allocations {
            tracing::info!(?allocation, "processing allocation");

            if allocation.amount == 0 {
                // Skip adding an empty note to the chain.
                continue;
            }

            let validator_base_denom = asset::REGISTRY.parse_denom(&allocation.denom).unwrap();

            let val = Value {
                amount: allocation.amount,
                asset_id: validator_base_denom.into(),
            };

            let blinding_factor_input = blake2b_simd::Params::default()
                .personal(b"genesis_note")
                .to_state()
                .update(&reward_counter.to_le_bytes())
                .finalize();
            reward_counter += 1;

            let destination = allocation.address;
            // build the note
            let note = Note::from_parts(
                *destination.diversifier(),
                *destination.transmission_key(),
                val,
                Fq::from_le_bytes_mod_order(blinding_factor_input.as_bytes()),
            )
            .unwrap();
            let commitment = note.commit();

            // append the note to the commitment tree
            note_commitment_tree.append(&commitment);

            tracing::debug!(?note, ?commitment);

            let esk = ka::Secret::new_from_field(Fr::one());
            let encrypted_note = note.encrypt(&esk);

            let note_data = NoteData {
                ephemeral_key: esk.diversified_public(&note.diversified_generator()),
                encrypted_note,
                // A transaction ID is either a hash of a transaction, or special data.
                // Special data is encoded with 23 leading 0 bytes, followed by a nonzero code byte,
                // followed by 8 data bytes.
                //
                // Transaction hashes can be confused with special data only if the transaction hash begins with 23 leading 0 bytes; this happens with probability 2^{-184}.
                //
                // Genesis transaction IDs use code 0x1.
                transaction_id: [[0; 23].to_vec(), [1].to_vec(), [0; 8].to_vec()]
                    .concat()
                    .try_into()
                    .unwrap(),
            };

            let denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .expect("genesis allocations must have valid denominations");

            // Accumulate the allocation amount into the supply updates for this denom.
            supply_updates.entry(denom.id()).or_insert((denom, 0)).1 += allocation.amount;

            // Keep track of the note so we can insert that as well for the sake of CompactBlock
            // it will need the position the note commitment tree
            let position = note_commitment_tree
                .bridges()
                .last()
                .map(|b| b.frontier().position().into())
                // If there are no bridges, the tree is empty
                .unwrap_or(0u64);
            let positioned_note = PositionedNoteData {
                position,
                data: note_data,
            };

            notes.push((commitment, positioned_note));
        }

        // Now that we've added all of the genesis notes to the NCT, compute the
        // resulting NCT anchor and save it in the database and in the JMT.
        let nct_anchor = note_commitment_tree.root2();
        let nct_bytes = bincode::serialize(&note_commitment_tree)?;

        // Save the NCT itself in the database ...
        query!(
            r#"
            INSERT INTO blobs (id, data) VALUES ('nct', $1)
            ON CONFLICT (id) DO UPDATE SET data = $1
            "#,
            &nct_bytes[..]
        )
        .execute(&mut dbtx)
        .await?;

        // ... and add its root to the public chain state ...
        let (jmt_root, tree_update_batch) = jmt::JellyfishMerkleTree::new(&self.private_reader)
            .put_value_set(
                // TODO: create a JmtKey enum, where each variant has
                // a different domain-separated hash
                vec![(
                    jellyfish::Key::NoteCommitmentAnchor.hash(),
                    nct_anchor.clone(),
                )],
                // height 0 for genesis
                0,
            )
            .await?;

        // ... and then write the resulting batch update to the backing store:
        jellyfish::DbTx(&mut dbtx)
            .write_node_batch(&tree_update_batch.node_batch)
            .await?;

        // As the very last step, compute the JMT root and return it as the apphash.
        let app_hash: [u8; 32] = jmt_root.to_vec().try_into().unwrap();

        // Insert the block into the DB
        query!(
            "INSERT INTO blocks (height, nct_anchor, app_hash) VALUES ($1, $2, $3)",
            0 as i64,
            &nct_anchor.to_bytes()[..],
            &app_hash[..]
        )
        .execute(&mut dbtx)
        .await?;

        // We might not have any allocations of some delegation tokens, but we should record the denoms.
        for genesis::ValidatorPower { validator, .. } in genesis_config.validators.iter() {
            let denom = validator.identity_key.delegation_token().denom();
            supply_updates.entry(denom.id()).or_insert((denom, 0)).1 += 0;
        }

        // write the token supplies to the database
        for (id, asset) in &supply_updates {
            query!(
                "INSERT INTO assets (asset_id, denom, total_supply)
            VALUES ($1, $2, $3)
            ON CONFLICT (asset_id) DO UPDATE SET denom=$2, total_supply=$3",
                &id.to_bytes()[..],
                asset.0.to_string(),
                i64::try_from(asset.1)?
            )
            .execute(&mut *dbtx)
            .await?;
        }
        for (commitment, positioned_note) in notes {
            query!(
                r#"
                INSERT INTO notes (
                    note_commitment,
                    ephemeral_key,
                    encrypted_note,
                    transaction_id,
                    position,
                    height
                ) VALUES ($1, $2, $3, $4, $5, $6)"#,
                &<[u8; 32]>::from(commitment)[..],
                &positioned_note.data.ephemeral_key.0[..],
                &positioned_note.data.encrypted_note[..],
                &positioned_note.data.transaction_id[..],
                i64::try_from(positioned_note.position)?,
                // height 0 for genesis
                0 as i64,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Finally, commit the transaction and then update subscribers
        // We've initialized the database for the first time, so replace
        // the default values as if we were loading while starting the node.
        dbtx.commit().await?;
        self.init_caches().await?;

        Ok(app_hash.to_vec())
    }

    /// Commits a block to the state, returning the new app hash.
    pub async fn commit_block(
        &self,
        block: PendingBlock,
        block_validator_set: &mut ValidatorSet,
    ) -> Result<Vec<u8>> {
        // TODO: batch these queries?
        let mut dbtx = self.pool.begin().await?;

        let nct_anchor = block.note_commitment_tree.root2();
        let nct_bytes = bincode::serialize(&block.note_commitment_tree)?;
        query!(
            r#"
            INSERT INTO blobs (id, data) VALUES ('nct', $1)
            ON CONFLICT (id) DO UPDATE SET data = $1
            "#,
            &nct_bytes[..]
        )
        .execute(&mut dbtx)
        .await?;

        let epoch = block.epoch.unwrap();
        let height = block.height.expect("height must be set");

        // The Jellyfish Merkle tree batches writes to its backing store, so we
        // first need to write the JMT kv pairs...
        let (jmt_root, tree_update_batch) = jmt::JellyfishMerkleTree::new(&self.private_reader)
            .put_value_set(
                // TODO: create a JmtKey enum, where each variant has
                // a different domain-separated hash
                vec![(
                    jellyfish::Key::NoteCommitmentAnchor.hash(),
                    nct_anchor.clone(),
                )],
                height,
            )
            .await?;
        // ... and then write the resulting batch update to the backing store:
        jellyfish::DbTx(&mut dbtx)
            .write_node_batch(&tree_update_batch.node_batch)
            .await?;

        // The app hash is the root of the Jellyfish Merkle Tree.  We save the
        // NCT anchor separately for convenience, but it's already included in
        // the JMT root.
        // TODO: no way to access the Diem HashValue as array, even though it's stored that way?
        let app_hash: [u8; 32] = jmt_root.to_vec().try_into().unwrap();

        query!(
            "INSERT INTO blocks (height, nct_anchor, app_hash) VALUES ($1, $2, $3)",
            i64::try_from(height)?,
            &nct_anchor.to_bytes()[..],
            &app_hash[..]
        )
        .execute(&mut dbtx)
        .await?;

        // Drop quarantined notes associated with a validator slashed in this block
        for note_commitment in block.reverting_notes {
            query!(
                "DELETE FROM quarantined_notes WHERE note_commitment = $1",
                &<[u8; 32]>::from(note_commitment)[..]
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Drop quarantined nullifiers from the main nullifier set if they were associated with a
        // validator slashed in this block (thus reverting their spend)
        for nullifier in block.reverting_nullifiers {
            // Forget about this nullifier, making the associated note spendable again
            query!(
                "DELETE FROM nullifiers WHERE nullifier = $1",
                &nullifier.to_bytes()[..]
            )
            .execute(&mut dbtx)
            .await?;

            // We have reverted this nullifier, so we can remove it from quarantine
            query!(
                "DELETE FROM quarantined_nullifiers WHERE nullifier = $1",
                &nullifier.to_bytes()[..]
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Add newly created notes into the chain state.
        for (note_commitment, positioned_note) in block.notes.into_iter() {
            query!(
                r#"
                INSERT INTO notes (
                    note_commitment,
                    ephemeral_key,
                    encrypted_note,
                    transaction_id,
                    position,
                    height
                ) VALUES ($1, $2, $3, $4, $5, $6)"#,
                &<[u8; 32]>::from(note_commitment)[..],
                &positioned_note.data.ephemeral_key.0[..],
                &positioned_note.data.encrypted_note[..],
                &positioned_note.data.transaction_id[..],
                i64::try_from(positioned_note.position)?,
                i64::try_from(height)?,
            )
            .execute(&mut dbtx)
            .await?;

            // If the note was previously quarantined, drop it from quarantine
            query!(
                "DELETE FROM quarantined_notes WHERE note_commitment = $1",
                &<[u8; 32]>::from(note_commitment)[..]
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Calculate the height at which notes quarantined in this block should unbond. If the
        // unbonding period or the epoch duration change, notes will unbond at the nearest epoch
        // boundary following this height.
        let unbonding_epochs = self
            .private_reader()
            .chain_params_rx()
            .borrow()
            .unbonding_epochs;
        let epoch_duration = block.epoch.as_ref().unwrap().duration;
        let unbonding_height = height + (epoch_duration * unbonding_epochs);

        // Add notes and nullifiers from transactions containing undelegations to a quarantine
        // queue, to be extracted when their unbonding period expires.
        for QuarantineGroup {
            validator_identity_key,
            notes,
            nullifiers,
        } in block.quarantine
        {
            // Quarantine all notes associated with this quarantine group
            for (&note_commitment, data) in notes.iter() {
                // Hold the note data in quarantine
                query!(
                    r#"
                    INSERT INTO quarantined_notes (
                        note_commitment,
                        ephemeral_key,
                        encrypted_note,
                        transaction_id,
                        unbonding_height,
                        validator_identity_key
                    ) VALUES ($1, $2, $3, $4, $5, $6)"#,
                    &<[u8; 32]>::from(note_commitment)[..],
                    &data.ephemeral_key.0[..],
                    &data.encrypted_note[..],
                    &data.transaction_id[..],
                    i64::try_from(unbonding_height)?,
                    &validator_identity_key.0.to_bytes()[..],
                )
                .execute(&mut dbtx)
                .await?;
            }

            // Quarantine all nullifiers associated with this quarantine group
            for &nullifier in nullifiers.iter() {
                let nullifier_bytes = &<[u8; 32]>::from(nullifier)[..];

                // Keep track of the nullifier associated with the block height
                query!(
                    r#"
                    INSERT INTO quarantined_nullifiers (nullifier, unbonding_height, validator_identity_key)
                    VALUES ($1, $2, $3)"#,
                    nullifier_bytes,
                    i64::try_from(unbonding_height)?,
                    &validator_identity_key.0.to_bytes()[..],
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Mark spent notes as spent.
        for nullifier in block.spent_nullifiers.into_iter() {
            query!(
                "INSERT INTO nullifiers VALUES ($1, $2)",
                &<[u8; 32]>::from(nullifier)[..],
                i64::try_from(height)?,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Save the validator set for this block to the database.
        let valid_anchors = &mut self.valid_anchors_tx.borrow().clone();

        // move to shielded pool handler
        if valid_anchors.len() >= NUM_RECENT_ANCHORS {
            valid_anchors.pop_back();
        }

        valid_anchors.push_front(nct_anchor.clone());

        tracing::debug!("calling block_validator_set.commit");
        block_validator_set.commit(&mut dbtx).await?;

        // Finally, commit the transaction and then update subscribers
        dbtx.commit().await?;

        // Next rate data is only available on the last block per epoch.
        // Fetch the next rate data from the DB
        // TODO: this should probably be stored on the JMT, but this works for now
        let next_rate_data = match epoch.end_height().value() == height {
            true => Some(self.private_reader().next_rate_data().await?),
            false => None,
        };

        // Errors in sends arise only if no one is listening -- not our problem.
        let _ = self.height_tx.send(height.try_into().unwrap());
        let _ = self.valid_anchors_tx.send(valid_anchors.clone());
        if let Some(next_rate_data) = next_rate_data {
            let _ = self.next_rate_data_tx.send(next_rate_data);
        }
        // chain_params_tx is a no-op, currently chain params don't change

        Ok(app_hash.to_vec())
    }
}
