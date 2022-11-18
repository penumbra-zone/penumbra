use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::StateTransaction;
use tendermint::abci;
use tracing::instrument;

use super::view::StateWriteExt as _;
use crate::Component;

use super::{execute, proposal::ProposalList};

pub struct Governance {}

#[async_trait]
impl Component for Governance {
    //#[instrument(name = "governance", skip(state, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {
        // Initialize the unfinished proposals tracking key in the JMT.
        // TODO: Replace with the new range queries in storage
        state
            .put_unfinished_proposals(ProposalList::default())
            .await;
    }

    #[instrument(name = "governance", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(state, _end_block))]
    async fn end_block(state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {
        // TODO: compute intermediate tallies at epoch boundaries (with threshold delegator voting)
        execute::enact_all_passed_proposals(state)
            .await
            .expect("failed to enact proposals");
        execute::enact_pending_parameter_changes(state)
            .await
            .expect("failed to enact parameter changes");
        execute::apply_proposal_refunds(state)
            .await
            .expect("failed to apply proposal refunds");
    }
}
