use std::{
    collections::{BTreeSet, HashMap},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::future::FutureExt;
use tower::Service;

use tendermint::abci::{response, Event, EventAttributeIndexExt, Request, Response};

use penumbra_crypto::{
    merkle, merkle::Frontier, merkle::TreeExt, note, Action, Nullifier, Transaction,
};
use tower_abci::BoxError;

const ABCI_INFO_VERSION: &'static str = env!("VERGEN_GIT_SEMVER");
const MAX_MERKLE_CHECKPOINTS: usize = 100;

/// The Penumbra ABCI application.
#[derive(Clone, Debug)]
pub struct App {
    store: HashMap<String, String>,
    height: u64,
    app_hash: [u8; 8],
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    nullifier_set: BTreeSet<Nullifier>,
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
            // unhandled messages
            Request::Flush => Response::Flush,
            Request::Echo(_) => Response::Echo(Default::default()),
            Request::InitChain(_) => Response::InitChain(Default::default()),
            Request::BeginBlock(_) => Response::BeginBlock(Default::default()),
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
            app_hash: [0; 8],
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS),
            // TODO: Store cached merkle root to prevent recomputing it - currently
            // this is happening for each spend (since we pass in the merkle_root when
            // verifying the spend proof).
            nullifier_set: BTreeSet::new(),
        }
    }
}

impl App {
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

    fn commit(&mut self) -> response::Commit {
        let retain_height = self.height as i64;
        // As in the other kvstore examples, just use store.len() as the "hash"
        self.app_hash = (self.store.len() as u64).to_be_bytes();
        self.height += 1;

        response::Commit {
            data: self.app_hash.to_vec().into(),
            retain_height,
        }
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
        }
        for commitment in note_commitments_to_add {
            self.note_commitment_tree.append(&commitment);
        }

        return true;
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
        assert_eq!(app.verify_transaction(transaction.unwrap()), false);
    }
}
