use std::{borrow::Borrow, collections::VecDeque};

use anyhow::Result;
use jmt::TreeWriterAsync;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::merkle::{self, TreeExt};
use penumbra_proto::Protobuf;
use penumbra_stake::{FundingStream, RateDataById, ValidatorStateName};
use sqlx::{query, Pool, Postgres};
use tendermint::block;
use tokio::sync::watch;

use super::jellyfish;
use crate::{
    genesis, pending_block::QuarantineGroup, validator_set::ValidatorSet, PendingBlock,
    NUM_RECENT_ANCHORS,
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
    /// Called by `state::new()` on init.
    pub(super) async fn init_caches(&self) -> Result<()> {
        let chain_params = self
            .private_reader
            .genesis_configuration()
            .await?
            .chain_params;
        let height = self.private_reader.height().await?;
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
    pub async fn commit_genesis(&self, genesis_config: &genesis::AppState) -> Result<()> {
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

        let mut next_rate_data = RateDataById::default();
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
                validator.sequence_number as i64,
                validator.name,
                validator.website,
                validator.description,
                power.value() as i64,
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

            next_rate_data.insert(
                validator.identity_key.clone(),
                penumbra_stake::RateData {
                    identity_key: validator.identity_key.clone(),
                    epoch_index: 1,
                    validator_reward_rate: 0,
                    validator_exchange_rate: 1_0000_0000,
                },
            );
        }

        let chain_params = genesis_config.chain_params.clone();
        // Finally, commit the transaction and then update subscribers
        dbtx.commit().await?;
        // Sends fail if every receiver has been dropped, which is not our problem.
        // We wrote these, so push updates to subscribers.
        let _ = self.chain_params_tx.send(chain_params);
        let _ = self.next_rate_data_tx.send(next_rate_data);
        // These haven't been set yet.
        // let _ = self.height_tx.send(height);
        // let _ = self.valid_anchors_tx.send(valid_anchors);

        Ok(())
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
            height as i64,
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
                positioned_note.position as i64,
                height as i64,
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
                    unbonding_height as i64,
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
                    unbonding_height as i64,
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
                height as i64,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Track the net change in delegations in this block.
        let epoch_index = block.epoch.clone().unwrap().index;
        for (identity_key, delegation_change) in &block_validator_set.delegation_changes {
            query!(
                "INSERT INTO delegation_changes VALUES ($1, $2, $3)",
                identity_key.encode_to_vec(),
                epoch_index as i64,
                delegation_change
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Save any new assets found in the block to the asset registry.
        for (id, asset) in &block_validator_set.supply_updates {
            query!(
                "INSERT INTO assets (asset_id, denom, total_supply)
                VALUES ($1, $2, $3)
                ON CONFLICT (asset_id) DO UPDATE SET denom=$2, total_supply=$3",
                &id.to_bytes()[..],
                asset.0.to_string(),
                asset.1 as i64
            )
            .execute(&mut dbtx)
            .await?;
        }

        if let (Some(base_rate_data), Some(rate_data)) = (
            block_validator_set.next_base_rate.clone(),
            block_validator_set.next_rates.clone(),
        ) {
            tracing::debug!(?base_rate_data, "Saving next base rate to the database");
            query!(
                "INSERT INTO base_rates VALUES ($1, $2, $3)",
                base_rate_data.epoch_index as i64,
                base_rate_data.base_reward_rate as i64,
                base_rate_data.base_exchange_rate as i64,
            )
            .execute(&mut dbtx)
            .await?;

            for rate in rate_data {
                query!(
                    "INSERT INTO validator_rates VALUES ($1, $2, $3, $4)",
                    rate.identity_key.encode_to_vec(),
                    rate.epoch_index as i64,
                    rate.validator_reward_rate as i64,
                    rate.validator_exchange_rate as i64,
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Handle adding newly added validators with default rates
        for v in &block_validator_set.new_validators {
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
                v.validator.identity_key.encode_to_vec(),
                v.validator.consensus_key.to_bytes(),
                v.validator.sequence_number as i64,
                v.validator.name,
                v.validator.website,
                v.validator.description,
                v.status.voting_power as i64,
                ValidatorStateName::Active.to_str().to_string(),
                Option::<i64>::None,
            )
            .execute(&mut dbtx)
            .await?;

            for FundingStream { address, rate_bps } in v.validator.funding_streams.as_ref() {
                query!(
                    "INSERT INTO validator_fundingstreams (
                        identity_key,
                        address,
                        rate_bps
                    ) VALUES ($1, $2, $3)",
                    v.validator.identity_key.encode_to_vec(),
                    address.to_string(),
                    *rate_bps as i32,
                )
                .execute(&mut dbtx)
                .await?;
            }

            // Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current and next epochs.
            for epoch in [epoch_index, epoch_index + 1] {
                query!(
                    "INSERT INTO validator_rates (
                    identity_key,
                    epoch,
                    validator_reward_rate,
                    validator_exchange_rate
                ) VALUES ($1, $2, $3, $4)",
                    v.validator.identity_key.encode_to_vec(),
                    epoch as i64,
                    0,
                    1_0000_0000i64, // 1 represented as 1e8
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Slashed validator states are saved at the end of the block.
        //
        // When the validator was slashed their rate was updated to incorporate
        // the slashing penalty and then their rate will be held constant, so
        // there is no need to take into account the slashing penalty here.
        for ik in block_validator_set.slashed_validators() {
            query!(
                "UPDATE validators SET validator_state=$1 WHERE identity_key = $2",
                ValidatorStateName::Slashed.to_str(),
                ik.borrow().encode_to_vec(),
            )
            .execute(&mut dbtx)
            .await?;
        }

        // This happens during every end_block. Most modifications to validator status occur
        // during end_epoch, and others (slashing) occur during begin_block, and both are
        // applied here.
        //
        // TODO: This isn't a differential update. This should be OK but is sub-optimal.
        for status in &block_validator_set.next_validator_statuses() {
            let (state_name, unbonding_epoch) = status.state.into();
            query!(
                    "UPDATE validators SET voting_power=$1, validator_state=$2, unbonding_epoch=$3 WHERE identity_key = $4",
                    status.voting_power as i64,
                    state_name.to_str(),
                    // unbonding_epoch column will be NULL if unbonding_epoch is None (i.e. the state is not unbonding)
                    unbonding_epoch.map(|i| i as i64),
                    status.identity_key.encode_to_vec(),
                )
                .execute(&mut dbtx)
                .await?;
        }

        let mut valid_anchors = self.valid_anchors_tx.borrow().clone();
        if valid_anchors.len() >= NUM_RECENT_ANCHORS {
            valid_anchors.pop_back();
        }
        valid_anchors.push_front(nct_anchor);
        let next_rate_data = block_validator_set.next_rates.as_ref().map(|next_rates| {
            next_rates
                .iter()
                .map(|rd| (rd.identity_key.clone(), rd.clone()))
                .collect::<RateDataById>()
        });

        // Finally, commit the transaction and then update subscribers
        dbtx.commit().await?;

        // Errors in sends arise only if no one is listening -- not our problem.
        let _ = self.height_tx.send(height.try_into().unwrap());
        let _ = self.valid_anchors_tx.send(valid_anchors);
        if let Some(next_rate_data) = next_rate_data {
            let _ = self.next_rate_data_tx.send(next_rate_data);
        }
        // chain_params_tx is a no-op, currently chain params don't change

        block_validator_set
            .commit_block(block.epoch.unwrap().clone())
            .await;

        Ok(app_hash.to_vec())
    }
}
