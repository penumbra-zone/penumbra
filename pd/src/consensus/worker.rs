use std::borrow::Borrow;

use anyhow::{anyhow, Result};
use futures::StreamExt;
use metrics::absolute_counter;
use penumbra_crypto::merkle::NoteCommitmentTree;
use penumbra_proto::Protobuf;
use penumbra_stake::Epoch;
use penumbra_transaction::Transaction;
use tendermint::abci::{self, ConsensusRequest as Request, ConsensusResponse as Response};
use tokio::sync::mpsc;
use tracing::Instrument;

use super::Message;
use crate::{
    components::validator_set::ValidatorSet, genesis, state, verify::StatelessTransactionExt,
    PendingBlock,
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
        // Because we want to be able to handle (re)loading the worker data after writing
        // the state snapshot in init_chain, we split out the real data loading into a single
        // Worker::load() method that can be called from both places. Since we need to initialize
        // the worker, though, fill in some garbage data that we'll immediately overwrite.
        // A more pedantically correct option would be to make everything Optional, but that
        // "contaminates" all of the other logic of the application to handle the initialization
        // special case.
        let reader = state.private_reader().clone();
        let mut worker = Self {
            state,
            queue,
            pending_block: None,
            note_commitment_tree: NoteCommitmentTree::new(0),
            block_validator_set: ValidatorSet::new(
                reader,
                Epoch {
                    index: 0,
                    duration: 1,
                },
            )
            .await?,
        };
        // If the database is still empty, this will still be garbage data, but we'll call
        // load() again when processing init_chain.
        worker.load().await?;

        Ok(worker)
    }

    /// Loads the worker's application data from the state.
    ///
    /// This is called in `new`, and also when (re)loading after writing the state snapshot from init_chain.
    async fn load(&mut self) -> Result<()> {
        let height = self.state.private_reader().height().await?.into();
        let epoch_duration = self
            .state
            .private_reader()
            .chain_params_rx()
            .borrow()
            .epoch_duration;
        let epoch = Epoch::from_height(height, epoch_duration);

        self.note_commitment_tree = self.state.private_reader().note_commitment_tree().await?;
        self.block_validator_set =
            ValidatorSet::new(self.state.private_reader().clone(), epoch).await?;
        self.pending_block = None;

        // Now (re)load the caches from the state writer:
        self.state.init_caches().await?;

        Ok(())
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

    /// Initializes the chain based on the genesis data.
    ///
    /// The genesis data is provided by tendermint, and is used to initialize
    /// the database.
    ///
    /// After the database has been initialized, the worker is reinstantiated,
    /// which will cause it to reload its state from the database, preparing it
    /// for the first block to begin.
    async fn init_chain(
        &mut self,
        init_chain: abci::request::InitChain,
    ) -> Result<abci::response::InitChain> {
        tracing::info!(?init_chain);
        // Note that errors cannot be handled in InitChain, the application must crash.
        let app_state: genesis::AppState = serde_json::from_slice(&init_chain.app_state_bytes)
            .expect("can parse app_state in genesis file");

        // Initialize the database with the app state.
        let app_hash = self.state.commit_genesis(&app_state).await?;

        // Reload the worker data from the database.
        self.load().await?;

        // Extract the Tendermint validators from the app state
        //
        // NOTE: we ignore the validators passed to InitChain.validators, and instead expect them
        // to be provided inside the initial app genesis state (`GenesisAppState`). Returning those
        // validators in InitChain::Response tells Tendermint that they are the initial validator
        // set. See https://docs.tendermint.com/master/spec/abci/abci.html#initchain
        let validators = self
            .block_validator_set
            .validators_info()
            .map(|v| {
                Ok(tendermint::abci::types::ValidatorUpdate {
                    pub_key: v.borrow().validator.consensus_key,
                    power: v.borrow().status.voting_power.try_into()?,
                })
            })
            .collect::<Result<Vec<tendermint::abci::types::ValidatorUpdate>>>()
            .expect("expected genesis state to reload correctly");

        Ok(abci::response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators,
            app_hash: app_hash.into(),
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

        self.block_validator_set.begin_block();

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
        // Use the current state of the validators in the state machine, not the ones in the db.
        let block_validators = self.block_validator_set.validators_info();
        // Verify the transaction is well-formed...
        let transaction = Transaction::decode(deliver_tx.tx)?
            // ... and that it is internally consistent ...
            .verify_stateless()?;
        // ... and that it is consistent with the existing chain state.
        let transaction = self
            .state
            .private_reader()
            .verify_stateful(transaction, block_validators)
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

        for v in &transaction.validator_definitions {
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

        // Validator updates need to be sent back to Tendermint during end_block, so we need to
        // tell the validator set the block has ended so it can resolve conflicts and prepare
        // data to commit.
        self.block_validator_set.end_block().await?;

        // Retrieve any changes that occurred during the block.
        let block_changes = self.block_validator_set.block_changes()?;

        // Send the next voting powers back to tendermint. This also
        // incorporates any newly added validators. Non-Active validators
        // will have their voting power reported to Tendermint set to 0.
        let validator_updates = block_changes.tm_validator_updates.clone();

        // Find out which validators were slashed in this block
        let slashed_validators = &block_changes.slashed_validators;

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

        // If we are at the end of an epoch, process changes for it
        if epoch.end_height().value() == height {
            self.end_epoch().await?;
        }

        tracing::debug!(?validator_updates, "setting validator updates");

        Ok(abci::response::EndBlock {
            validator_updates,
            consensus_param_updates: None,
            events: Vec::new(),
        })
    }

    /// Process the state transitions for the end of an epoch.
    async fn end_epoch(&mut self) -> Result<()> {
        tracing::debug!("consensus: end_epoch");
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
        let prev_epoch = &self.block_validator_set.epoch();
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
        self.block_validator_set
            .end_epoch(current_epoch.clone())
            .await?;

        let epoch_changes = self.block_validator_set.epoch_changes()?;
        // Add reward notes to the pending block.
        for reward_note in &epoch_changes.reward_notes {
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
