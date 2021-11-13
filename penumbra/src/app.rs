use std::{
    collections::BTreeSet,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Context as _;
use bytes::Bytes;
use futures::{executor::block_on, future::FutureExt};
use rand_core::OsRng;
use sqlx::{query_as, Pool, Postgres};
use tendermint::abci::{
    request::{self, BeginBlock, EndBlock},
    response, Request, Response,
};
use tower::Service;

use penumbra_crypto::{
    merkle, merkle::Frontier, merkle::TreeExt, note, Action, Nullifier, Transaction,
};
use tower_abci::BoxError;
use tracing::instrument;

use crate::{
    dbschema::{self, PenumbraStateFragment, PenumbraTransaction},
    dbutils::{db_commit_block, db_connection},
    genesis::GenesisNotes,
    state::PendingBlock,
};

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");

/// The Penumbra ABCI application.
#[derive(Debug)]
pub struct App {
    db: Pool<Postgres>,

    height: u64,
    app_hash: [u8; 32],

    /// Should be written to the database after every block commit.
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,

    /// We want to prevent two transactions from spending the same note in the
    /// same block.  Our only control over whether transactions will appear in a
    /// block is in `CheckTx`, which gates access to the entire mempool, so we
    /// want to enforce that no two transactions in the mempool spend the same
    /// note.
    ///
    /// To do this, we add a mempool transaction's nullifiers to this set in
    /// `CheckTx` and remove them when we see they've been committed to a block
    /// (in `Commit`).  This means that if Tendermint pulls transactions from
    /// the mempool as part of default block proposer logic, no conflicting
    /// transactions can appear.
    ///
    /// However, it doesn't prevent a malicious validator from proposing
    /// conflicting transactions, so we need to ensure (in `DeliverTx`) that we
    /// ignore invalid transactions.
    mempool_nullifiers: BTreeSet<Nullifier>,

    /// Contains all queued state changes for the duration of a block.  This is
    /// set to Some at the beginning of BeginBlock and consumed (and reset to
    /// None) in Commit.
    pending_block: Option<PendingBlock>,
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

            // TODO
            Request::CheckTx(_) => Response::CheckTx(Default::default()),

            Request::Commit => Response::Commit(self.commit()),
            Request::BeginBlock(begin) => Response::BeginBlock(self.begin_block(begin)),
            Request::DeliverTx(deliver_tx) => Response::DeliverTx(self.deliver_tx(deliver_tx.tx)),
            Request::EndBlock(end) => Response::EndBlock(self.end_block(end)),

            // Called only once for network genesis, i.e. when the application block height is 0.
            Request::InitChain(init_chain) => Response::InitChain(self.init_genesis(init_chain)),

            // unhandled messages
            Request::Flush => Response::Flush,
            Request::Echo(_) => Response::Echo(Default::default()),
            Request::ListSnapshots => Response::ListSnapshots(Default::default()),
            Request::OfferSnapshot(_) => Response::OfferSnapshot(Default::default()),
            Request::LoadSnapshotChunk(_) => Response::LoadSnapshotChunk(Default::default()),
            Request::ApplySnapshotChunk(_) => Response::ApplySnapshotChunk(Default::default()),
        };
        tracing::info!(?rsp);
        async move { Ok(rsp) }.boxed()
    }
}

impl App {
    /// Load the application state from the database.
    #[instrument]
    pub async fn load() -> Result<Self, anyhow::Error> {
        let db = db_connection().await.context("Could not open database")?;

        let note_commitment_tree = if let Some(dbschema::KeyedBlob { data, .. }) =
            query_as::<_, dbschema::KeyedBlob>("SELECT id, data FROM blobs WHERE id = 'nct';")
                .fetch_optional(&mut db.acquire().await?)
                .await?
        {
            bincode::deserialize(&data).context("Could not parse saved note commitment tree")?
        } else {
            merkle::BridgeTree::new(0)
        };

        // TODO load app_hash and height

        Ok(Self {
            db,
            app_hash: [0; 32],
            height: 0,
            note_commitment_tree,
            mempool_nullifiers: Default::default(),
            pending_block: None,
        })
    }

    #[instrument]
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
        self.pending_block = Some(current_block);
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

    #[instrument]
    fn info(&self) -> response::Info {
        response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: 1,
            last_block_height: self.height as i64,
            last_block_app_hash: self.app_hash.to_vec().into(),
        }
    }

    #[instrument]
    fn query(&self, query: Bytes) -> response::Query {
        // TODO: implement (#23)
        Default::default()
    }

    #[instrument]
    fn begin_block(&mut self, begin: BeginBlock) -> response::BeginBlock {
        self.pending_block = Some(PendingBlock::default());
        response::BeginBlock::default()
    }

    #[instrument]
    fn deliver_tx(&mut self, tx: Bytes) -> response::DeliverTx {
        // TODO: implement (#135)

        // This should accumulate data from `tx` into `self.pending_block`

        Default::default()
    }

    #[instrument]
    fn end_block(&mut self, end: EndBlock) -> response::EndBlock {
        // TODO: here's where we process validator changes
        response::EndBlock::default()
    }

    /// Commit the queued state transitions.
    #[instrument]
    fn commit(&mut self) -> response::Commit {
        tracing::info!("committing pending changes to database");
        let pending_block =
            std::mem::replace(&mut self.pending_block, None).expect("we must have pending changes");

        // Update local NCT.
        for note_commitment in &pending_block.note_commitments {
            self.note_commitment_tree.append(&note_commitment);
        }

        // TODO: remove nullifiers from mempool_nullifiers ?

        // TODO: pass note_commitment_tree to db_commit_block ?

        let retain_height = self.height as i64;
        self.app_hash = self.note_commitment_tree.root2().to_bytes();
        block_on(db_commit_block(
            &self.db,
            pending_block,
            retain_height,
            self.app_hash,
        ));

        self.height += 1;

        response::Commit {
            data: self.app_hash.to_vec().into(),
            retain_height,
        }
    }

    /// Verifies a transaction and if it verifies, updates the node state.
    ///
    /// TODO: split into stateless and stateful parts.
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
                    /*
                    if self.nullifier_set.contains(&inner.body.nullifier.clone())
                        || nullifiers_to_add.contains(&inner.body.nullifier.clone())
                    {
                        return false;
                    }
                    */

                    // Queue up the state changes.
                    nullifiers_to_add.insert(inner.body.nullifier.clone());
                }
            }
        }

        // 3. Update node state. (TODO: remove and put in commit ?)
        for nf in nullifiers_to_add {
            //self.nullifier_set.insert(nf);
            // xx add nullifier set storage in db?
        }
        for commitment in note_commitments_to_add {
            self.note_commitment_tree.append(&commitment);
            // xx add row in transactions table
        }

        true
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
        /*
        // TODO: restore this test after writing a state facade (?)

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
        */
    }
}
