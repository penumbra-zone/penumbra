use std::{
    collections::{BTreeSet, HashMap},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::{executor::block_on, future::FutureExt};
use rand_core::OsRng;
use sqlx::{query_as, Pool, Postgres};
use tendermint::abci::{request, response, Event, EventAttributeIndexExt, Request, Response};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tower::Service;

use penumbra_crypto::{
    merkle, merkle::Frontier, merkle::TreeExt, note, Action, Nullifier, Transaction,
};
use penumbra_proto::transaction;
use penumbra_proto::wallet::{
    wallet_server::Wallet, CompactBlock, CompactBlockRangeRequest, TransactionByNoteRequest,
};
use tower_abci::BoxError;

use crate::{
    dbschema::{PenumbraStateFragment, PenumbraTransaction},
    dbutils::{db_commit_block, db_connection},
    genesis::GenesisNotes,
    state::PendingBlock,
};

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");
const MAX_MERKLE_CHECKPOINTS: usize = 100;

/// The Penumbra ABCI application.
#[derive(Debug)]
pub struct App {
    store: HashMap<String, String>,
    height: u64,

    /// The `app_hash` is the hash of the note commitment tree.
    app_hash: [u8; 32],
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    nullifier_set: BTreeSet<Nullifier>,

    // When `BeginBlock` (or `InitChain` for block 0) is called, we set `current_block`
    // to hold a struct which holds all the queued-up state changes for the duration of the block.
    // We commit these changes to the database and set it back to `None` when we `Commit`.
    current_block: Option<PendingBlock>,
}

async fn get_database_connection() -> Pool<Postgres> {
    db_connection().await.expect("")
}

impl Service<Request> for App {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Response, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        tracing::info!(?req);

        let rsp = match req {
            // handled messages
            Request::Info(_) => Response::Info(self.info()),
            Request::Query(query) => Response::Query(self.query(query.data)),
            Request::DeliverTx(deliver_tx) => Response::DeliverTx(self.deliver_tx(deliver_tx.tx)),
            Request::Commit => Response::Commit(self.commit()),
            Request::BeginBlock(_) => Response::BeginBlock(self.begin_block()),

            // Called only once for network genesis, i.e. when the application block height is 0.
            Request::InitChain(init_chain) => Response::InitChain(self.init_genesis(init_chain)),

            // unhandled messages
            Request::Flush => Response::Flush,
            Request::Echo(_) => Response::Echo(Default::default()),
            Request::CheckTx(_) => Response::CheckTx(Default::default()),
            Request::EndBlock(_) => Response::EndBlock(Default::default()),
            Request::ListSnapshots => Response::ListSnapshots(Default::default()),
            Request::OfferSnapshot(_) => Response::OfferSnapshot(Default::default()),
            Request::LoadSnapshotChunk(_) => Response::LoadSnapshotChunk(Default::default()),
            Request::ApplySnapshotChunk(_) => Response::ApplySnapshotChunk(Default::default()),
        };
        tracing::info!(?rsp);
        async move { Ok(rsp) }.boxed()
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            store: HashMap::default(),
            height: 0,
            app_hash: [0; 32],
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS),
            // TODO: Store cached merkle root to prevent recomputing it - currently
            // this is happening for each spend (since we pass in the merkle_root when
            // verifying the spend proof).
            nullifier_set: BTreeSet::new(),
            current_block: None,
        }
    }
}

impl App {
    fn init_genesis(&mut self, init_chain: request::InitChain) -> response::InitChain {
        let mut current_block = PendingBlock::default();
        tracing::info!("performing genesis for chain_id: {}", init_chain.chain_id);

        // Note that errors cannot be handled in InitChain, the application must crash.
        let genesis: GenesisNotes = serde_json::from_slice(&init_chain.app_state_bytes)
            .expect("can parse app_state in genesis file");

        // Create genesis transaction and update database table `transactions`.
        let mut genesis_tx_builder =
            Transaction::genesis_build_with_root(self.note_commitment_tree.root2());

        for note in genesis.notes() {
            genesis_tx_builder.add_output(&mut OsRng, note);
        }
        let genesis_tx = genesis_tx_builder
            .set_chain_id(init_chain.chain_id)
            .finalize(&mut OsRng)
            .expect("can form genesis transaction");

        // Now add the transaction and its note fragments to the pending state changes.
        current_block.add_transaction(genesis_tx);
        tracing::info!("successfully loaded all genesis notes");

        // xx Correct/Necessary to commit here or will tendermint after InitGenesis?
        self.current_block = Some(current_block);
        self.commit();
        let initial_application_hash = self.app_hash.to_vec().into();
        tracing::info!(
            "initial app_hash at genesis: {:?}",
            initial_application_hash
        );

        response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators: init_chain.validators,
            app_hash: initial_application_hash,
        }
    }

    fn info(&self) -> response::Info {
        response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: 1,
            last_block_height: self.height as i64,
            last_block_app_hash: self.app_hash.to_vec().into(),
        }
    }

    fn query(&self, query: Bytes) -> response::Query {
        let key = String::from_utf8(query.to_vec()).unwrap();
        let (value, log) = match self.store.get(&key) {
            Some(value) => (value.clone(), "exists".to_string()),
            None => ("".to_string(), "does not exist".to_string()),
        };

        response::Query {
            log,
            key: key.into_bytes().into(),
            value: value.into_bytes().into(),
            ..Default::default()
        }
    }

    fn deliver_tx(&mut self, tx: Bytes) -> response::DeliverTx {
        let tx = String::from_utf8(tx.to_vec()).unwrap();
        let tx_parts = tx.split('=').collect::<Vec<_>>();
        let (key, value) = match (tx_parts.get(0), tx_parts.get(1)) {
            (Some(key), Some(value)) => (*key, *value),
            _ => (tx.as_ref(), tx.as_ref()),
        };
        self.store.insert(key.to_string(), value.to_string());

        response::DeliverTx {
            events: vec![Event::new(
                "app",
                vec![
                    ("key", key).index(),
                    ("index_key", "index is working").index(),
                    ("noindex_key", "index is working").no_index(),
                ],
            )],
            ..Default::default()
        }
    }

    /// Commit the queued state transitions.
    fn commit(&mut self) -> response::Commit {
        tracing::info!("committing pending changes to database");
        let current_block =
            std::mem::replace(&mut self.current_block, None).expect("we must have pending changes");

        // Update local NCT.
        for note_commitment in &current_block.note_commitments {
            self.note_commitment_tree.append(&note_commitment);
        }

        // TODO: Update local nullifier set.

        let retain_height = self.height as i64;
        self.app_hash = self.note_commitment_tree.root2().to_bytes();
        let db_connection = block_on(get_database_connection());
        block_on(db_commit_block(
            db_connection,
            current_block,
            retain_height,
            self.app_hash,
        ));

        self.height += 1;

        response::Commit {
            data: self.app_hash.to_vec().into(),
            retain_height,
        }
    }

    fn begin_block(&mut self) -> response::BeginBlock {
        self.current_block = Some(PendingBlock::default());
        response::BeginBlock::default()
    }

    /// Verifies a transaction and if it verifies, updates the node state.
    pub fn verify_transaction(&mut self, transaction: Transaction) -> bool {
        // 1. Check binding signature.
        if !transaction.verify_binding_sig() {
            return false;
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut nullifiers_to_add = BTreeSet::<Nullifier>::new();
        let mut note_commitments_to_add = Vec::<note::Commitment>::new();

        for action in transaction.transaction_body().actions {
            match action {
                Action::Output(inner) => {
                    if !inner.body.proof.verify(
                        inner.body.value_commitment,
                        inner.body.note_commitment,
                        inner.body.ephemeral_key,
                    ) {
                        return false;
                    }

                    // Queue up the state changes.
                    note_commitments_to_add.push(inner.body.note_commitment);
                }
                Action::Spend(inner) => {
                    if !inner.verify_auth_sig() {
                        return false;
                    }

                    if !inner.body.proof.verify(
                        self.note_commitment_tree.root2(),
                        inner.body.value_commitment,
                        inner.body.nullifier.clone(),
                        inner.body.rk,
                    ) {
                        return false;
                    }

                    // Check nullifier is not already in the nullifier set OR
                    // has been revealed already in this transaction.
                    if self.nullifier_set.contains(&inner.body.nullifier.clone())
                        || nullifiers_to_add.contains(&inner.body.nullifier.clone())
                    {
                        return false;
                    }

                    // Queue up the state changes.
                    nullifiers_to_add.insert(inner.body.nullifier.clone());
                }
            }
        }

        // 3. Update node state.
        for nf in nullifiers_to_add {
            self.nullifier_set.insert(nf);
            // xx add nullifier set storage in db?
        }
        for commitment in note_commitments_to_add {
            self.note_commitment_tree.append(&commitment);
            // xx add row in transactions table
        }

        true
    }
}

/// The Penumbra wallet service.
pub struct WalletApp {
    db_pool: Pool<Postgres>,
}

impl WalletApp {
    pub fn new() -> WalletApp {
        WalletApp {
            db_pool: block_on(get_database_connection()),
        }
    }
}

#[tonic::async_trait]
impl Wallet for WalletApp {
    type CompactBlockRangeStream = ReceiverStream<Result<CompactBlock, Status>>;

    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        let mut p = self
            .db_pool
            .acquire()
            .await
            .map_err(|_| tonic::Status::unavailable("server error"))?;
        let request = request.into_inner();
        let start_height = request.start_height;
        let end_height = request.end_height;

        if end_height < start_height {
            return Err(tonic::Status::failed_precondition(
                "end height must be greater than start height",
            ));
        }

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            for block_height in start_height..=end_height {
                let rows = query_as::<_, PenumbraStateFragment>(
                    r#"
SELECT note_commitment, ephemeral_key, note_ciphertext FROM notes
WHERE transaction_id IN (select id from transactions where block_id IN
(SELECT id FROM blocks WHERE height = $1)
)
"#,
                )
                .bind(block_height)
                .fetch_all(&mut p)
                .await
                .expect("if no results will return empty state fragments");

                let block = CompactBlock {
                    height: block_height,
                    fragment: rows.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
                };
                tracing::info!("sending block response: {:?}", block);
                tx.send(Ok(block.clone())).await.unwrap();
            }
        });

        Ok(tonic::Response::new(Self::CompactBlockRangeStream::new(rx)))
    }

    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<transaction::Transaction>, Status> {
        let mut p = self
            .db_pool
            .acquire()
            .await
            .map_err(|_| tonic::Status::unavailable("server error"))?;

        let note_commitment = request.into_inner().cm;
        let rows = query_as::<_, PenumbraTransaction>(
            r#"
SELECT transactions.transaction FROM transactions
JOIN notes ON transactions.id = (SELECT transaction_id FROM notes WHERE note_commitment=$1
)
"#,
        )
        .bind(note_commitment)
        .fetch_one(&mut p)
        .await
        .map_err(|_| tonic::Status::not_found("transaction not found"))?;

        let transaction = penumbra_crypto::Transaction::try_from(&rows.transaction[..])
            .map_err(|_| tonic::Status::data_loss("transaction not well formed"))?;
        let protobuf_transaction: transaction::Transaction = transaction.into();

        Ok(tonic::Response::new(protobuf_transaction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::Zero;
    use rand_core::OsRng;

    use penumbra_crypto::{keys::SpendKey, memo::MemoPlaintext, Fq, Note, Value};

    #[test]
    fn test_transaction_verification_fails_for_dummy_merkle_tree() {
        let mut app = App::default();

        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let mut merkle_siblings = Vec::new();
        for _i in 0..merkle::DEPTH {
            merkle_siblings.push(note::Commitment(Fq::zero()))
        }
        let dummy_merkle_path: merkle::Path = (merkle::DEPTH, merkle_siblings);

        let value_to_send = Value {
            amount: 10,
            asset_id: b"pen".as_ref().into(),
        };
        let dummy_note = Note::new(
            *dest.diversifier(),
            *dest.transmission_key(),
            value_to_send,
            Fq::zero(),
        )
        .expect("transmission key is valid");

        let transaction = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .set_chain_id("Pen".to_string())
            .add_output(
                &mut rng,
                &dest,
                value_to_send,
                MemoPlaintext::default(),
                ovk_sender,
            )
            .add_spend(&mut rng, sk_sender, dummy_merkle_path, dummy_note, 0.into())
            .finalize(&mut rng);

        // The merkle path is invalid, so this transaction should not verify.
        assert!(!app.verify_transaction(transaction.unwrap()));
    }
}
