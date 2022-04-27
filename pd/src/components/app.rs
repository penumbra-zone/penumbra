use std::str::FromStr;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jmt::{RootHash, Version};
use penumbra_chain::{genesis, params::ChainParams};
use penumbra_component::Component;
use penumbra_stake::Epoch;
use penumbra_storage::{State, StateExt, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};
use tendermint::Time;
use tracing::instrument;

use super::{IBCComponent, ShieldedPool, Staking};

/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is also a [`Component`], but as the top-level component,
/// it constructs the others and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    state: State,
    shielded_pool: ShieldedPool,
    ibc: IBCComponent,
    staking: Staking,
}

impl App {
    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> Result<(RootHash, Version)> {
        // Commit the pending writes, clearing the state.
        let (root_hash, version) = self.state.lock().await.commit(storage).await?;
        tracing::debug!(?root_hash, version, "finished committing state");
        // Now re-instantiate all of the components:
        self.staking = Staking::new(self.state.clone()).await;
        self.ibc = IBCComponent::new(self.state.clone()).await;
        self.shielded_pool = ShieldedPool::new(self.state.clone()).await;

        Ok((root_hash, version))
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        self.staking.tm_validator_updates().await
    }
}

#[async_trait]
impl Component for App {
    #[instrument(skip(state))]
    async fn new(state: State) -> Self {
        let staking = Staking::new(state.clone()).await;
        let ibc = IBCComponent::new(state.clone()).await;
        let shielded_pool = ShieldedPool::new(state.clone()).await;

        Self {
            state,
            shielded_pool,
            staking,
            ibc,
        }
    }

    #[instrument(skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        self.state
            .put_chain_params(app_state.chain_params.clone())
            .await;
        // TODO: do we actually need to store the app state here?
        self.state
            .put_domain(b"genesis/app_state".into(), app_state.clone())
            .await;
        // The genesis block height is 0
        self.state.put_block_height(0).await;

        self.staking.init_chain(app_state).await;
        self.ibc.init_chain(app_state).await;

        // Shielded pool always executes last.
        self.shielded_pool.init_chain(app_state).await;
    }

    #[instrument(skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) {
        // store the block height
        self.state
            .put_block_height(begin_block.header.height.into())
            .await;
        // store the block time
        self.state
            .put_block_timestamp(begin_block.header.time)
            .await;

        self.staking.begin_block(begin_block).await;
        self.ibc.begin_block(begin_block).await;
        // Shielded pool always executes last.
        self.shielded_pool.begin_block(begin_block).await;
    }

    #[instrument(skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        Staking::check_tx_stateless(tx)?;
        IBCComponent::check_tx_stateless(tx)?;
        ShieldedPool::check_tx_stateless(tx)?;
        Ok(())
    }

    #[instrument(skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        self.staking.check_tx_stateful(tx).await?;
        self.ibc.check_tx_stateful(tx).await?;

        // Shielded pool always executes last.
        self.shielded_pool.check_tx_stateful(tx).await?;
        Ok(())
    }

    #[instrument(skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        self.staking.execute_tx(tx).await;
        self.ibc.execute_tx(tx).await;
        // Shielded pool always executes last.
        self.shielded_pool.execute_tx(tx).await;
    }

    #[instrument(skip(self, end_block))]
    async fn end_block(&mut self, end_block: &abci::request::EndBlock) {
        self.staking.end_block(end_block).await;
        self.ibc.end_block(end_block).await;

        // Shielded pool always executes last.
        self.shielded_pool.end_block(end_block).await;
    }
}

/// This trait provides read and write access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
#[async_trait]
pub trait View: StateExt {
    /// Gets the chain parameters from the JMT.
    async fn get_chain_params(&self) -> Result<ChainParams> {
        self.get_domain(b"chain_params".into())
            .await?
            .ok_or_else(|| anyhow!("Missing ChainParams"))
    }

    /// Writes the provided chain parameters to the JMT.
    async fn put_chain_params(&self, params: ChainParams) {
        self.put_domain(b"chain_params".into(), params).await
    }

    /// Gets the current epoch for the chain.
    async fn get_current_epoch(&self) -> Result<Epoch> {
        let block_height = self.get_block_height().await?;
        Ok(Epoch::from_height(
            block_height,
            self.get_epoch_duration().await?,
        ))
    }

    /// Gets the epoch duration for the chain.
    async fn get_epoch_duration(&self) -> Result<u64> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        self.get_chain_params()
            .await
            .map(|params| params.epoch_duration)
    }

    /// Gets the chain ID.
    async fn get_chain_id(&self) -> Result<String> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        self.get_chain_params().await.map(|params| params.chain_id)
    }

    /// Gets the current block height from the JMT
    async fn get_block_height(&self) -> Result<u64> {
        let height_bytes: u64 = self
            .get_proto(b"block_height".into())
            .await?
            .ok_or_else(|| anyhow!("Missing block_height"))?;

        Ok(height_bytes)
    }

    /// Writes the block height to the JMT
    async fn put_block_height(&self, height: u64) {
        self.put_proto(b"block_height".into(), height).await
    }

    /// Gets the current block timestamp from the JMT
    async fn get_block_timestamp(&self) -> Result<Time> {
        let timestamp_string: String = self
            .get_proto(b"block_timestamp".into())
            .await?
            .ok_or_else(|| anyhow!("Missing block_timestamp"))?;

        Ok(Time::from_str(&timestamp_string).unwrap())
    }

    /// Writes the block timestamp to the JMT
    async fn put_block_timestamp(&self, timestamp: Time) {
        self.put_proto(b"block_timestamp".into(), timestamp.to_rfc3339())
            .await
    }

    /// Checks a provided chain_id against the chain state.
    ///
    /// Passes through if the provided chain_id is empty or matches, and
    /// otherwise errors.
    async fn check_chain_id(&self, provided: &str) -> Result<(), tonic::Status> {
        let chain_id = self
            .get_chain_id()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;
        if provided.is_empty() || provided == chain_id {
            Ok(())
        } else {
            Err(tonic::Status::failed_precondition(format!(
                "provided chain_id {} does not match chain_id {}",
                provided, chain_id
            )))
        }
    }
}

impl<T: StateExt> View for T {}
