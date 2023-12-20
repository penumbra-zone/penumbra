use std::sync::Arc;

use anyhow::Result;
use cnidarium::StateWrite;
use ibc_types::{
    core::client::Height, lightclients::tendermint::ConsensusState as TendermintConsensusState,
};
use tendermint::abci;
use tracing::instrument;

use cnidarium_component::ChainStateReadExt;

use crate::component::{client::StateWriteExt as _, client_counter::ClientCounter};

pub struct IBCComponent {}

impl IBCComponent {
    #[instrument(name = "ibc", skip(state, app_state))]
    pub async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&()>) {
        match app_state {
            Some(_) => state.put_client_counter(ClientCounter(0)),
            None => { /* perform upgrade specific check */ }
        }
    }

    #[instrument(name = "ibc", skip(state, begin_block))]
    pub async fn begin_block<S: StateWrite + ChainStateReadExt>(
        state: &mut S,
        begin_block: &abci::request::BeginBlock,
    ) {
        // In BeginBlock, we want to save a copy of our consensus state to our
        // own state tree, so that when we get a message from our
        // counterparties, we can verify that they are committing the correct
        // consensus states for us to their state tree.
        let commitment_root: Vec<u8> = begin_block.header.app_hash.clone().into();
        let cs = TendermintConsensusState::new(
            ibc_types::core::commitment::MerkleRoot {
                hash: commitment_root,
            },
            begin_block.header.time,
            begin_block.header.next_validators_hash,
        );

        // Currently, we don't use a revision number, because we don't have
        // any further namespacing of blocks than the block height.
        let height = Height::new(
            state
                .get_revision_number()
                .await
                .expect("must be able to get revision number in begin block"),
            begin_block.header.height.into(),
        )
        .expect("block height cannot be zero");

        state.put_penumbra_consensus_state(height, cs);
    }

    #[instrument(name = "ibc", skip(_state, _end_block))]
    pub async fn end_block<S: StateWrite + 'static>(
        mut _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "ibc", skip(_state))]
    pub async fn end_epoch<S: StateWrite + 'static>(mut _state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}
