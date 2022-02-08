use anyhow::{anyhow, Result};
use metrics::absolute_counter;
use penumbra_crypto::{asset, merkle::NoteCommitmentTree};
use penumbra_proto::Protobuf;
use penumbra_stake::{
    ValidatorState, ValidatorStatus, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, ConsensusRequest as Request, ConsensusResponse as Response};
use tokio::sync::mpsc;
use tracing::Instrument;

use super::Message;
use crate::{genesis, state, verify::StatelessTransactionExt, PendingBlock};

pub struct Worker {
    state: state::Writer,
    queue: mpsc::Receiver<Message>,
    // todo: split up and modularize
    pending_block: Option<PendingBlock>,
    note_commitment_tree: NoteCommitmentTree,
}

impl Worker {
    pub async fn new(state: state::Writer, queue: mpsc::Receiver<Message>) -> Result<Self> {
        let note_commitment_tree = state.private_reader().note_commitment_tree().await?;

        Ok(Self {
            state,
            queue,
            pending_block: None,
            note_commitment_tree,
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
            genesis_block
                .supply_updates
                .entry(denom.id())
                .or_insert((denom, 0))
                .1 += allocation.amount;
        }

        // We might not have any allocations of delegation tokens, but we should record the denoms.
        for genesis::ValidatorPower { validator, .. } in app_state.validators.iter() {
            let denom = validator.identity_key.delegation_token().denom();
            genesis_block
                .supply_updates
                .entry(denom.id())
                .or_insert((denom, 0));
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

        // Update metrics.
        let current_nullifier_count = self.state.private_reader().nullifier_count().await?;
        absolute_counter!("node_spent_nullifiers_total", current_nullifier_count);

        assert!(self.pending_block.is_none());
        self.pending_block = Some(PendingBlock::new(self.note_commitment_tree.clone()));

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
        // Verify the transaction is well-formed...
        let transaction = Transaction::decode(deliver_tx.tx)?
            // ... and that it is internally consistent ...
            .verify_stateless()?;
        // ... and that it is consistent with the existing chain state.
        let transaction = self
            .state
            .private_reader()
            .verify_stateful(transaction)
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

        tracing::debug!(?height, ?epoch, end_height = ?epoch.end_height());

        // If we are at the end of an epoch, process changes for it
        if epoch.end_height().value() == height {
            self.end_epoch().await?;
        }

        // TODO: revert notes and nullifiers immediately from slashed validators in this block

        // TODO: right now we are not writing the updated voting power from validator statuses
        // back to tendermint, so that we can see how the statuses are computed without risking
        // halting the testnet. in the future we want to add code here to send the next voting
        // powers back to tendermint.
        Ok(Default::default())
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
        let prev_epoch = pending_block
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

        // TODO (optimization): batch these queries
        let current_base_rate = reader.base_rate_data(current_epoch.index).await?;
        let current_rates = reader.rate_data(current_epoch.index).await?;

        let mut staking_token_supply = reader
            .asset_lookup(*STAKING_TOKEN_ASSET_ID)
            .await?
            .map(|info| info.total_supply)
            .unwrap();

        // steps (foreach validator):
        // - get the total token supply for the validator's delegation tokens
        // - process the updates to the token supply:
        //   - collect all delegations occurring in previous epoch and apply them (adds to supply);
        //   - collect all undelegations started in previous epoch and apply them (reduces supply);
        // - feed the updated (current) token supply into current_rates.voting_power()
        // - persist both the current voting power and the current supply
        //

        /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
        /// 3bps -> 11% return over 365 epochs, why not
        const BASE_REWARD_RATE: u64 = 3_0000;

        let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        let mut next_rates = Vec::new();
        let mut next_validator_statuses = Vec::new();

        // this is a bit complicated: because we're in the EndBlock phase, and the
        // delegations in this block have not yet been committed, we have to combine
        // the delegations in pending_block with the ones already committed to the
        // state. otherwise the delegations committed in the epoch threshold block
        // would be lost.
        let mut delegation_changes = reader.delegation_changes(prev_epoch.index).await?;
        for (id_key, delta) in &pending_block.delegation_changes {
            *delegation_changes.entry(id_key.clone()).or_insert(0) += delta;
        }

        for current_rate in &current_rates {
            let identity_key = current_rate.identity_key.clone();

            let funding_streams = reader.funding_streams(identity_key.clone()).await?;
            let next_rate = current_rate.next(&next_base_rate, funding_streams.as_ref());

            // TODO: if a validator isn't part of the consensus set, should we ignore them
            // and not update their rates?
            let delegation_delta = delegation_changes.get(&identity_key).unwrap_or(&0i64);

            let delegation_amount = delegation_delta.abs() as u64;
            let unbonded_amount = current_rate.unbonded_amount(delegation_amount);

            let mut delegation_token_supply = reader
                .asset_lookup(identity_key.delegation_token().id())
                .await?
                .map(|info| info.total_supply)
                .unwrap_or(0);

            if *delegation_delta > 0 {
                // net delegation: subtract the unbonded amount from the staking token supply
                staking_token_supply = staking_token_supply.checked_sub(unbonded_amount).unwrap();
                delegation_token_supply = delegation_token_supply
                    .checked_add(delegation_amount)
                    .unwrap();
            } else {
                // net undelegation: add the unbonded amount to the staking token supply
                staking_token_supply = staking_token_supply.checked_add(unbonded_amount).unwrap();
                delegation_token_supply = delegation_token_supply
                    .checked_sub(delegation_amount)
                    .unwrap();
            }

            // update the delegation token supply
            pending_block.supply_updates.insert(
                identity_key.delegation_token().id(),
                (
                    identity_key.delegation_token().denom(),
                    delegation_token_supply,
                ),
            );

            let voting_power = next_rate.voting_power(delegation_token_supply, &next_base_rate);
            let next_status = ValidatorStatus {
                identity_key,
                voting_power,
                // TODO: this state needs to be set correctly based on current state and any changes
                // within the current block. This will be fixed by #375.
                state: ValidatorState::Active,
            };

            // distribute validator commission
            for stream in funding_streams {
                let commission_reward_amount = stream.reward_amount(
                    delegation_token_supply,
                    &next_base_rate,
                    &current_base_rate,
                );

                pending_block.add_validator_reward_note(commission_reward_amount, stream.address);
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?next_status);

            next_rates.push(next_rate);
            next_validator_statuses.push(next_status);
        }

        tracing::debug!(?staking_token_supply);

        pending_block.next_rates = Some(next_rates);
        pending_block.next_base_rate = Some(next_base_rate);
        pending_block.next_validator_statuses = Some(next_validator_statuses);
        pending_block.supply_updates.insert(
            *STAKING_TOKEN_ASSET_ID,
            (STAKING_TOKEN_DENOM.clone(), staking_token_supply),
        );

        // TODO: later, set the EndBlock response to add validators
        // at the epoch boundary

        Ok(())
    }

    async fn commit(&mut self) -> Result<abci::response::Commit> {
        let pending_block = self
            .pending_block
            .take()
            .expect("pending_block must be Some in Commit");

        // Pull the updated note commitment tree, for use in the next block.
        self.note_commitment_tree = pending_block.note_commitment_tree.clone();

        let app_hash = self.state.commit_block(pending_block).await?;

        tracing::info!(app_hash = ?hex::encode(&app_hash), "finished block commit");

        Ok(abci::response::Commit {
            data: app_hash.into(),
            retain_height: 0u32.into(),
        })
    }
}
