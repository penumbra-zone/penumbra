use std::borrow::Borrow;

use anyhow::{anyhow, Result};
use futures::StreamExt;
use metrics::absolute_counter;
use penumbra_crypto::{asset, merkle::NoteCommitmentTree};
use penumbra_proto::Protobuf;
use penumbra_transaction::Transaction;
use tendermint::abci::{self, ConsensusRequest as Request, ConsensusResponse as Response};
use tokio::sync::mpsc;
use tracing::Instrument;

use super::Message;
use crate::{
    genesis, state, validator_set::ValidatorSet, verify::StatelessTransactionExt, PendingBlock,
};

pub struct Worker {
    state: state::Writer,
    queue: mpsc::Receiver<Message>,
    // todo: split up and modularize
    pending_block: Option<PendingBlock>,
    block_validator_set: ValidatorSet,
    note_commitment_tree: NoteCommitmentTree,
}

impl Worker {
    pub async fn new(state: state::Writer, queue: mpsc::Receiver<Message>) -> Result<Self> {
        let note_commitment_tree = state.private_reader().note_commitment_tree().await?;
        let block_validator_set = ValidatorSet::new(state.private_reader().clone()).await?;

        Ok(Self {
            state,
            queue,
            pending_block: None,
            note_commitment_tree,
            block_validator_set,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(Message {
            req,
            rsp_sender,
            span,
        }) = self.queue.recv().await
        {
            // The send only fails if the receiver was dropped, which happens
            // if the caller didn't propagate the message back to tendermint
            // for some reason -- but that's not our problem.
            let _ = rsp_sender.send(match req {
                Request::InitChain(init_chain) => Response::InitChain(
                    self.init_chain(init_chain)
                        .instrument(span)
                        .await
                        .expect("init_chain must succeed"),
                ),
                Request::BeginBlock(begin_block) => Response::BeginBlock(
                    self.begin_block(begin_block)
                        .instrument(span)
                        .await
                        .expect("begin_block must succeed"),
                ),
                Request::DeliverTx(deliver_tx) => {
                    Response::DeliverTx(match self.deliver_tx(deliver_tx).instrument(span).await {
                        Ok(()) => abci::response::DeliverTx::default(),
                        Err(e) => abci::response::DeliverTx {
                            code: 1,
                            log: e.to_string(),
                            ..Default::default()
                        },
                    })
                }
                Request::EndBlock(end_block) => Response::EndBlock(
                    self.end_block(end_block)
                        .instrument(span)
                        .await
                        .expect("end_block must succeed"),
                ),
                Request::Commit => Response::Commit(
                    self.commit()
                        .instrument(span)
                        .await
                        .expect("commit must succeed"),
                ),
            });
        }
        Ok(())
    }

    async fn init_chain(
        &mut self,
        init_chain: abci::request::InitChain,
    ) -> Result<abci::response::InitChain> {
        tracing::info!(?init_chain);
        // Note that errors cannot be handled in InitChain, the application must crash.
        let app_state: genesis::AppState = serde_json::from_slice(&init_chain.app_state_bytes)
            .expect("can parse app_state in genesis file");

        // Initialize the database with the app state.
        self.state.commit_genesis(&app_state).await?;

        // Now start building the genesis block:
        self.note_commitment_tree = NoteCommitmentTree::new(0);
        let mut genesis_block = PendingBlock::new(self.note_commitment_tree.clone());
        genesis_block.set_height(0, app_state.chain_params.epoch_duration);

        // Create a genesis transaction to record genesis notes.
        // TODO: eliminate this (#374)
        // replace with methods on pendingblock for genesis notes that handle
        // supply tracking
        let mut tx_builder = Transaction::genesis_builder();

        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            tx_builder.add_output(allocation.note().expect("genesis allocations are valid"));

            let denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .expect("genesis allocations must have valid denominations");

            // Accumulate the allocation amount into the supply updates for this denom.
            self.block_validator_set
                .update_supply_for_denom(denom, allocation.amount);
        }

        // We might not have any allocations of delegation tokens, but we should record the denoms.
        for genesis::ValidatorPower { validator, .. } in app_state.validators.iter() {
            let denom = validator.identity_key.delegation_token().denom();
            self.block_validator_set.update_supply_for_denom(denom, 0);
        }

        let genesis_tx = tx_builder
            .set_chain_id(init_chain.chain_id)
            .finalize()
            .expect("can form genesis transaction");
        let verified_transaction = crate::verify::mark_genesis_as_verified(genesis_tx);

        // Now add the transaction and its note fragments to the pending state changes.
        genesis_block.add_transaction(verified_transaction);

        // Commit the genesis block to the state
        self.pending_block = Some(genesis_block);
        let app_hash = self.commit().await?.data;

        // Extract the Tendermint validators from the genesis app state
        //
        // NOTE: we ignore the validators passed to InitChain.validators, and instead expect them
        // to be provided inside the initial app genesis state (`GenesisAppState`). Returning those
        // validators in InitChain::Response tells Tendermint that they are the initial validator
        // set. See https://docs.tendermint.com/master/spec/abci/abci.html#initchain
        let validators = app_state
            .validators
            .iter()
            .map(|genesis::ValidatorPower { validator, power }| {
                tendermint::abci::types::ValidatorUpdate {
                    pub_key: validator.consensus_key,
                    power: *power,
                }
            })
            .collect();

        Ok(abci::response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators,
            app_hash,
        })
    }

    async fn begin_block(
        &mut self,
        begin_block: abci::request::BeginBlock,
    ) -> Result<abci::response::BeginBlock> {
        tracing::debug!(?begin_block);

        let block_metrics = self.state.private_reader().metrics().await?;
        absolute_counter!("node_spent_nullifiers_total", block_metrics.nullifier_count);
        absolute_counter!("node_notes_total", block_metrics.note_count);

        assert!(self.pending_block.is_none());
        self.pending_block = Some(PendingBlock::new(self.note_commitment_tree.clone()));

        let slashing_penalty = self
            .state
            .private_reader()
            .chain_params_rx()
            .borrow()
            .slashing_penalty;

        // For each validator identified as byzantine by tendermint, update its
        // status to be slashed.
        for evidence in begin_block.byzantine_validators.iter() {
            let ck = tendermint::PublicKey::from_raw_ed25519(&evidence.validator.address)
                .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey from tendermint"))
                .unwrap();

            self.block_validator_set
                .slash_validator(&ck, slashing_penalty)?;
        }

        Ok(Default::default())
    }

    /// Perform full transaction validation via `DeliverTx`.
    ///
    /// State changes are only applied for valid transactions. Invalid transaction are ignored.
    ///
    /// We must perform all checks again here even though they are performed in `CheckTx`, as a
    /// Byzantine node may propose a block containing double spends or other disallowed behavior,
    /// so it is not safe to assume all checks performed in `CheckTx` were done.
    async fn deliver_tx(&mut self, deliver_tx: abci::request::DeliverTx) -> Result<()> {
        let block_validators = self.state.private_reader().validator_info(true).await?;
        // Verify the transaction is well-formed...
        let transaction = Transaction::decode(deliver_tx.tx)?
            // ... and that it is internally consistent ...
            .verify_stateless()?;
        // ... and that it is consistent with the existing chain state.
        let transaction = self
            .state
            .private_reader()
            .verify_stateful(transaction, &block_validators)
            .await?;

        let mut conflicts = self
            .pending_block
            .as_ref()
            .unwrap()
            .spent_nullifiers
            .intersection(&transaction.spent_nullifiers);

        if let Some(conflict) = conflicts.next() {
            return Err(anyhow!(
                "nullifier {:?} is already spent in the pending block",
                conflict
            ));
        }

        for v in &transaction.new_validators {
            self.block_validator_set.add_validator_definition(v.clone());
        }

        // Tell the validator set about the delegation changes in this transaction
        self.block_validator_set
            .update_delegations(&transaction.delegation_changes);

        self.pending_block
            .as_mut()
            .unwrap()
            .add_transaction(transaction);

        Ok(())
    }

    async fn end_block(
        &mut self,
        end_block: abci::request::EndBlock,
    ) -> Result<abci::response::EndBlock> {
        tracing::debug!(?end_block);

        let reader = self.state.private_reader();

        let pending_block = self
            .pending_block
            .as_mut()
            .expect("pending block must be Some in EndBlock");

        let height = end_block
            .height
            .try_into()
            .expect("height should be nonnegative");
        let epoch = pending_block.set_height(
            height,
            self.state
                .private_reader()
                .chain_params_rx()
                .borrow()
                .epoch_duration,
        );

        // The block validator set also needs to know the epoch to perform rate calculations.

        tracing::debug!(?height, ?epoch, end_height = ?epoch.end_height());

        // Find out which validators were slashed in this block
        let slashed_validators = &self.block_validator_set.slashed_validators;

        // Immediately revert notes and nullifiers immediately from slashed validators in this block
        let (mut slashed_notes, mut slashed_nullifiers) = (
            reader.quarantined_notes(None, Some(slashed_validators.iter())),
            reader.quarantined_nullifiers(None, Some(slashed_validators.iter())),
        );
        while let Some(result) = slashed_notes.next().await {
            pending_block.reverting_notes.insert(result?.1); // insert the commitment
        }
        while let Some(result) = slashed_nullifiers.next().await {
            pending_block.reverting_nullifiers.insert(result?.1); // insert the nullifier
        }
        drop(slashed_notes);
        drop(slashed_nullifiers);

        // Validator updates need to be sent back to Tendermint during end_block, so we need to
        // tell the validator set the block has ended so it can resolve conflicts and prepare
        // data to commit.
        self.block_validator_set.end_block(epoch.clone()).await?;

        // If we are at the end of an epoch, process changes for it
        if epoch.end_height().value() == height {
            self.end_epoch().await?;
        }

        // Send the next voting powers back to tendermint. This also
        // incorporates any newly added validators.
        let validator_updates = self.block_validator_set.tm_validator_updates.clone();

        Ok(abci::response::EndBlock {
            validator_updates,
            consensus_param_updates: None,
            events: Vec::new(),
        })
    }

    /// Process the state transitions for the end of an epoch.
    async fn end_epoch(&mut self) -> Result<()> {
        let reader = self.state.private_reader();

        let pending_block = self
            .pending_block
            .as_mut()
            .expect("pending block must be Some in EndBlock");

        let height = pending_block
            .height
            .expect("height must already have been set");

        // We've finished processing the last block of `epoch`, so we've
        // crossed the epoch boundary, and (prev | current | next) are:
        let prev_epoch = &self
            .block_validator_set
            .epoch
            .clone()
            .expect("epoch must already have been set");
        let current_epoch = prev_epoch.next();
        let next_epoch = current_epoch.next();

        tracing::info!(
            ?height,
            ?prev_epoch,
            ?current_epoch,
            ?next_epoch,
            "crossed epoch boundary, processing rate updates"
        );
        metrics::increment_counter!("epoch");

        // Find all the validators which are *not* slashed in this block
        let well_behaved_validators = &self
            .block_validator_set
            .unslashed_validators()
            .map(|v| v.borrow().identity_key.clone())
            .collect::<Vec<_>>();

        // Process unbonding notes and nullifiers for this epoch
        let (mut unbonding_notes, mut unbonding_nullifiers) = (
            reader.quarantined_notes(Some(height), Some(well_behaved_validators.iter())),
            reader.quarantined_nullifiers(Some(height), Some(well_behaved_validators.iter())),
        );
        while let Some(result) = unbonding_notes.next().await {
            let (_, commitment, data) = result?;
            pending_block.add_note(commitment, data);
        }
        while let Some(result) = unbonding_nullifiers.next().await {
            pending_block.unbonding_nullifiers.insert(result?.1); // insert the nullifier
        }
        drop(unbonding_notes);
        drop(unbonding_nullifiers);

        // Tell the validator set that the epoch is changing so it can prepare to commit.
        self.block_validator_set.end_epoch().await?;

        // Add reward notes to the pending block.
        for reward_note in &self.block_validator_set.reward_notes {
            pending_block.add_validator_reward_note(reward_note.0, reward_note.1);
        }

        Ok(())
    }

    async fn commit(&mut self) -> Result<abci::response::Commit> {
        let pending_block = self
            .pending_block
            .take()
            .expect("pending_block must be Some in Commit");

        // Pull the updated note commitment tree, for use in the next block.
        self.note_commitment_tree = pending_block.note_commitment_tree.clone();

        let app_hash = self
            .state
            .commit_block(pending_block, &mut self.block_validator_set)
            .await?;

        tracing::info!(app_hash = ?hex::encode(&app_hash), "finished block commit");

        Ok(abci::response::Commit {
            data: app_hash.into(),
            retain_height: 0u32.into(),
        })
    }
}
