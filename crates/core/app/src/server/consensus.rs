use anyhow::Result;

use cnidarium::Storage;
use tendermint::abci::Event;
use tendermint::v0_37::abci::{
    request, response, ConsensusRequest as Request, ConsensusResponse as Response,
};
use tokio::sync::mpsc;
use tower::BoxError;
use tower_actor::Message;
use tracing::Instrument;

use crate::app::App;

pub struct Consensus {
    queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    storage: Storage,
    app: App,
}

pub type ConsensusService = tower_actor::Actor<Request, Response, BoxError>;

fn trace_events(events: &[Event]) {
    for event in events {
        let span = tracing::debug_span!("event", kind = ?event.kind);
        span.in_scope(|| {
            for attr in &event.attributes {
                tracing::debug!(
                    k = %String::from_utf8_lossy(attr.key_bytes()),
                    v = %String::from_utf8_lossy(attr.value_bytes()),
                );
            }
        })
    }
}

impl Consensus {
    const QUEUE_SIZE: usize = 10;

    pub fn new(storage: Storage) -> ConsensusService {
        tower_actor::Actor::new(Self::QUEUE_SIZE, |queue: _| {
            Consensus::new_inner(storage, queue).run()
        })
    }

    fn new_inner(
        storage: Storage,
        queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    ) -> Self {
        let app = App::new(storage.latest_snapshot());

        Self {
            queue,
            storage,
            app,
        }
    }

    async fn run(mut self) -> Result<(), tower::BoxError> {
        while let Some(Message {
            req,
            rsp_sender,
            span,
        }) = self.queue.recv().await
        {
            // The send only fails if the receiver was dropped, which happens
            // if the caller didn't propagate the message back to tendermint
            // for some reason -- but that's not our problem.
            let _ = rsp_sender.send(Ok(match req {
                Request::InitChain(init_chain) => Response::InitChain(
                    self.init_chain(init_chain)
                        .instrument(span)
                        .await
                        .expect("init_chain must succeed"),
                ),
                Request::PrepareProposal(proposal) => Response::PrepareProposal(
                    self.prepare_proposal(proposal)
                        .instrument(span)
                        .await
                        .expect("prepare proposal must succeed"),
                ),
                Request::ProcessProposal(proposal) => Response::ProcessProposal(
                    self.process_proposal(proposal)
                        .instrument(span)
                        .await
                        .expect("process proposal must succeed"),
                ),
                Request::BeginBlock(begin_block) => Response::BeginBlock(
                    self.begin_block(begin_block)
                        .instrument(span)
                        .await
                        .expect("begin_block must succeed"),
                ),
                Request::DeliverTx(deliver_tx) => {
                    Response::DeliverTx(self.deliver_tx(deliver_tx).instrument(span.clone()).await)
                }
                Request::EndBlock(end_block) => {
                    Response::EndBlock(self.end_block(end_block).instrument(span).await)
                }
                Request::Commit => Response::Commit(
                    self.commit()
                        .instrument(span)
                        .await
                        .expect("commit must succeed"),
                ),
            }));
        }
        Ok(())
    }

    /// Initializes the chain based on the genesis data.
    ///
    /// The genesis data is provided by tendermint, and is used to initialize
    /// the database.
    async fn init_chain(&mut self, init_chain: request::InitChain) -> Result<response::InitChain> {
        // Note that errors cannot be handled in InitChain, the application must crash.
        let app_state: crate::genesis::AppState =
            serde_json::from_slice(&init_chain.app_state_bytes)
                .expect("can parse app_state in genesis file");

        self.app.init_chain(&app_state).await;

        // Extract the Tendermint validators from the app state
        //
        // NOTE: we ignore the validators passed to InitChain.validators, and instead expect them
        // to be provided inside the initial app genesis state (`GenesisAppState`). Returning those
        // validators in InitChain::Response tells Tendermint that they are the initial validator
        // set. See https://docs.tendermint.com/master/spec/abci/abci.html#initchain
        let validators = self.app.cometbft_validator_updates();

        let app_hash = match &app_state {
            crate::genesis::AppState::Checkpoint(h) => {
                tracing::info!(?h, "genesis state is a checkpoint");
                // If we're starting from a checkpoint, we just need to forward the app hash
                // back to CometBFT.
                self.storage.latest_snapshot().root_hash().await?
            }
            crate::genesis::AppState::Content(_) => {
                tracing::info!("genesis state is a full configuration");
                // Check that we haven't got a duplicated InitChain message for some reason:
                if self.storage.latest_version() != u64::MAX {
                    anyhow::bail!("database already initialized");
                }
                // Note: App::commit resets internal components, so we don't need to do that ourselves.
                self.app.commit(self.storage.clone()).await
            }
        };

        tracing::info!(
            consensus_params = ?init_chain.consensus_params,
            ?validators,
            app_hash = ?app_hash,
            "finished init_chain"
        );

        Ok(response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators,
            app_hash: app_hash.0.to_vec().try_into()?,
        })
    }

    async fn prepare_proposal(
        &mut self,
        proposal: request::PrepareProposal,
    ) -> Result<response::PrepareProposal> {
        tracing::info!(height = ?proposal.height, proposer = ?proposal.proposer_address, "preparing proposal");
        // We prepare a proposal against an isolated fork of the application state.
        let mut tmp_app = App::new(self.storage.latest_snapshot());
        // Once we are done, we discard it so that the application state doesn't get corrupted
        // if another round of consensus is required because the proposal fails to finalize.
        Ok(tmp_app.prepare_proposal(proposal).await)
    }

    async fn process_proposal(
        &mut self,
        proposal: request::ProcessProposal,
    ) -> Result<response::ProcessProposal> {
        tracing::info!(height = ?proposal.height, proposer = ?proposal.proposer_address, proposal_hash = %proposal.hash, "processing proposal");
        // We process the proposal in an isolated state fork. Eventually, we should cache this work and
        // re-use it when processing a `FinalizeBlock` message (starting in `0.38.x`).
        let mut tmp_app = App::new(self.storage.latest_snapshot());
        Ok(tmp_app.process_proposal(proposal).await)
    }

    async fn begin_block(
        &mut self,
        begin_block: request::BeginBlock,
    ) -> Result<response::BeginBlock> {
        // We don't need to print the block height, because it will already be
        // included in the span modeling the abci request handling.
        tracing::info!(time = ?begin_block.header.time, "beginning block");

        let events = self.app.begin_block(&begin_block).await;

        Ok(response::BeginBlock { events })
    }

    async fn deliver_tx(&mut self, deliver_tx: request::DeliverTx) -> response::DeliverTx {
        // Unlike the other messages, DeliverTx is fallible, so
        // inspect the response to report errors.
        let rsp = self.app.deliver_tx_bytes(deliver_tx.tx.as_ref()).await;

        match rsp {
            Ok(events) => {
                trace_events(&events);
                response::DeliverTx {
                    events,
                    ..Default::default()
                }
            }
            Err(e) => {
                tracing::info!(?e, "deliver_tx failed");
                response::DeliverTx {
                    code: 1.into(),
                    // Use the alternate format specifier to include the chain of error causes.
                    log: format!("{e:#}"),
                    ..Default::default()
                }
            }
        }
    }

    async fn end_block(&mut self, end_block: request::EndBlock) -> response::EndBlock {
        let latest_state_version = self.storage.latest_version();
        tracing::info!(height = ?end_block.height, ?latest_state_version, "ending block");
        if latest_state_version >= end_block.height as u64 {
            tracing::warn!(
                %latest_state_version,
                %end_block.height,
                "chain state version is ahead of the block height, this is an unexpected corruption of chain state"
            );
        }
        let events = self.app.end_block(&end_block).await;
        trace_events(&events);

        // Set `tm_validator_updates` to the complete set of
        // validators and voting power. This must be the last step performed,
        // after all voting power calculations and validator state transitions have
        // been completed.
        let validator_updates = self.app.cometbft_validator_updates();

        tracing::debug!(
            ?validator_updates,
            "sending validator updates to tendermint"
        );

        response::EndBlock {
            validator_updates,
            consensus_param_updates: None,
            events,
        }
    }

    async fn commit(&mut self) -> Result<response::Commit> {
        let app_hash = self.app.commit(self.storage.clone()).await;
        tracing::info!(?app_hash, "committed block");

        Ok(response::Commit {
            data: app_hash.0.to_vec().into(),
            retain_height: 0u32.into(),
        })
    }
}
