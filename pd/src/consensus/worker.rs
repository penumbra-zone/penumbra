use std::sync::Arc;

use anyhow::{anyhow, Result};

use penumbra_proto::Protobuf;

use penumbra_chain::genesis;
use penumbra_storage::Storage;
use penumbra_transaction::Transaction;
use tendermint::abci::{self, ConsensusRequest as Request, ConsensusResponse as Response};
use tokio::sync::mpsc;
use tracing::{instrument, Instrument};

use super::Message;
use crate::App;

pub struct Worker {
    queue: mpsc::Receiver<Message>,
    storage: Storage,
    app: App,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "consensus::Worker::new")]
    pub async fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> Result<Self> {
        let app = App::new(storage.state());

        Ok(Self {
            queue,
            storage,
            app,
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
                    // Unlike the other messages, DeliverTx is fallible, so
                    // we use a wrapper function to catch bubbled errors.
                    let rsp = self.deliver_tx(deliver_tx).instrument(span.clone()).await;
                    span.in_scope(|| {
                        Response::DeliverTx(match rsp {
                            Ok(events) => {
                                tracing::info!("deliver_tx succeeded");
                                abci::response::DeliverTx {
                                    events,
                                    ..Default::default()
                                }
                            }
                            Err(e) => {
                                tracing::info!(?e, "deliver_tx failed");
                                abci::response::DeliverTx {
                                    code: 1,
                                    log: e.to_string(),
                                    ..Default::default()
                                }
                            }
                        })
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
    async fn init_chain(
        &mut self,
        init_chain: abci::request::InitChain,
    ) -> Result<abci::response::InitChain> {
        tracing::info!(?init_chain);
        // Note that errors cannot be handled in InitChain, the application must crash.
        let app_state: genesis::AppState = serde_json::from_slice(&init_chain.app_state_bytes)
            .expect("can parse app_state in genesis file");

        // Check that we haven't got a duplicated InitChain message for some reason:
        if self.storage.latest_version() != u64::MAX {
            return Err(anyhow!("database already initialized"));
        }
        self.app.init_chain(&app_state).await;

        // Extract the Tendermint validators from the app state
        //
        // NOTE: we ignore the validators passed to InitChain.validators, and instead expect them
        // to be provided inside the initial app genesis state (`GenesisAppState`). Returning those
        // validators in InitChain::Response tells Tendermint that they are the initial validator
        // set. See https://docs.tendermint.com/master/spec/abci/abci.html#initchain
        let validators = self.app.tendermint_validator_updates();

        // Note: App::commit resets internal components, so we don't need to do that ourselves.
        let app_hash = self
            .app
            .commit(self.storage.clone())
            .await
            .expect("must be able to commit state");

        tracing::info!(
            consensus_params = ?init_chain.consensus_params,
            ?validators,
            app_hash = ?app_hash,
            "finished init_chain"
        );

        Ok(abci::response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators,
            app_hash: app_hash.0.to_vec().into(),
        })
    }

    async fn begin_block(
        &mut self,
        begin_block: abci::request::BeginBlock,
    ) -> Result<abci::response::BeginBlock> {
        let events = self.app.begin_block(&begin_block).await;
        Ok(abci::response::BeginBlock { events })
    }

    /// Perform full transaction validation via `DeliverTx`.
    ///
    /// This wrapper function allows us to bubble up errors and then finally
    /// convert to an ABCI error code in one place.
    async fn deliver_tx(
        &mut self,
        deliver_tx: abci::request::DeliverTx,
    ) -> Result<Vec<abci::Event>> {
        // TODO: do we still need this wrapper function?
        // Verify the transaction is well-formed...
        let transaction = Arc::new(Transaction::decode(deliver_tx.tx)?);
        // ... then execute the transaction.
        self.app.deliver_tx(transaction).await
    }

    async fn end_block(
        &mut self,
        end_block: abci::request::EndBlock,
    ) -> Result<abci::response::EndBlock> {
        let events = self.app.end_block(&end_block).await;

        // Set `tm_validator_updates` to the complete set of
        // validators and voting power. This must be the last step performed,
        // after all voting power calculations and validator state transitions have
        // been completed.
        let validator_updates = self.app.tendermint_validator_updates();

        tracing::debug!(
            ?validator_updates,
            "sending validator updates to tendermint"
        );

        Ok(abci::response::EndBlock {
            validator_updates,
            consensus_param_updates: None,
            events,
        })
    }

    async fn commit(&mut self) -> Result<abci::response::Commit> {
        let app_hash = self
            .app
            .commit(self.storage.clone())
            .await
            .expect("commit must succeed")
            .0
            .to_vec();

        Ok(abci::response::Commit {
            data: app_hash.into(),
            retain_height: 0u32.into(),
        })
    }
}
