use anyhow::{anyhow, Result};

use penumbra_proto::Protobuf;

use penumbra_chain::genesis;
use penumbra_component::{Component, Context};
use penumbra_storage::Storage;
use penumbra_transaction::Transaction;
use tendermint::{
    abci::{self, ConsensusRequest as Request, ConsensusResponse as Response},
    block,
};
use tokio::sync::{mpsc, watch};
use tracing::{instrument, Instrument};

use super::Message;
use crate::App;

pub struct Worker {
    queue: mpsc::Receiver<Message>,
    height_tx: watch::Sender<block::Height>,
    storage: Storage,
    app: App,
}

impl Worker {
    #[instrument(skip(storage, queue, height_tx), name = "consensus::Worker::new")]
    pub async fn new(
        storage: Storage,
        queue: mpsc::Receiver<Message>,
        height_tx: watch::Sender<block::Height>,
    ) -> Result<Self> {
        let app = App::new(storage.clone()).await;

        Ok(Self {
            queue,
            height_tx,
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
                    let ctx = Context::new();
                    let rsp = self.deliver_tx(deliver_tx).instrument(span.clone()).await;
                    span.in_scope(|| {
                        Response::DeliverTx(match rsp {
                            Ok(()) => {
                                tracing::info!("deliver_tx succeeded");
                                abci::response::DeliverTx {
                                    events: ctx.into_events(),
                                    ..Default::default()
                                }
                            }
                            Err(e) => {
                                tracing::info!(?e, "deliver_tx failed");
                                abci::response::DeliverTx {
                                    code: 1,
                                    log: e.to_string(),
                                    events: ctx.into_events(),
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
        if self.storage.latest_version().await?.is_some() {
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
        let (app_hash, _) = self.app.commit(self.storage.clone()).await?;

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
        let ctx = Context::new();
        self.app.begin_block(&begin_block).await;
        Ok(abci::response::BeginBlock {
            events: ctx.into_events(),
        })
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
        let transaction = Transaction::decode(deliver_tx.tx)?;
        // ... and statelessly valid...
        App::check_tx_stateless(&transaction)?;
        // ... and statefully valid.
        self.app.check_tx_stateful(&transaction).await?;
        // Now execute the transaction. It's important to panic on error here, since if
        // we fail to execute the transaction here, it's because of an internal
        // error and we may have left the chain in an inconsistent state.
        self.app.execute_tx(&transaction).await;
        Ok(())
    }

    async fn end_block(
        &mut self,
        end_block: abci::request::EndBlock,
    ) -> Result<abci::response::EndBlock> {
        let ctx = Context::new();
        self.app.end_block(&end_block).await;

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
            events: ctx.into_events(),
        })
    }

    async fn commit(&mut self) -> Result<abci::response::Commit> {
        // Begin sidecar code

        // Note: App::commit resets internal components, so we don't need to do that ourselves.
        let (jmt_root, _) = self.app.commit(self.storage.clone()).await?;
        let app_hash = jmt_root.0.to_vec();
        let _ = self.height_tx.send(
            self.storage
                .latest_version()
                .await?
                .expect("just committed version")
                .try_into()
                .unwrap(),
        );

        tracing::info!(app_hash = ?hex::encode(&app_hash), "finished block commit");

        Ok(abci::response::Commit {
            data: app_hash.into(),
            retain_height: 0u32.into(),
        })
    }
}
