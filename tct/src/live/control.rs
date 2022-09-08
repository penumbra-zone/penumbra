use std::sync::Arc;

use axum::{
    extract::Query,
    http::StatusCode,
    routing::{post, MethodRouter},
    Json, Router,
};

use parking_lot::Mutex;
use rand::{seq::SliceRandom, Rng};
use serde_json::json;
use tokio::sync::watch;

use crate::{
    builder::{block, epoch},
    Commitment, Tree, Witness,
};

/// An [`axum`] [`Router`] that serves a `POST` endpoint for updating the [`Tree`].
///
/// Queries taking arguments pass arguments via URL parameters. Results are returned in JSON format,
/// with [`StatusCode::BAD_REQUEST`] being returned if the operation failed (i.e. an `Err` variant
/// was returned).
pub fn control<R: Rng + Send + 'static>(
    rng: R,
    tree: Arc<watch::Sender<Tree>>,
    max_witnesses: Option<usize>,
) -> Router {
    // The rng is shared between all methods
    let rng = Arc::new(Mutex::new(rng));

    Router::new()
        .route("/new", new(tree.clone()))
        .route("/insert", insert(rng.clone(), tree.clone(), max_witnesses))
        .route("/forget", forget(rng.clone(), tree.clone()))
        .route(
            "/insert-block-root",
            insert_block_root(rng.clone(), tree.clone()),
        )
        .route("/end-block", end_block(tree.clone()))
        .route("/insert-epoch-root", insert_epoch_root(rng, tree.clone()))
        .route("/end-epoch", end_epoch(tree))
}

fn new(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    post(|| async move {
        tree.send_modify(|tree| {
            *tree = Tree::new();
        })
    })
}

fn insert<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
    max_witnesses: Option<usize>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct Insert {
        witness: Witness,
        commitment: Option<Commitment>,
    }

    post(
        move |Query(Insert {
                  witness,
                  commitment,
              }): Query<Insert>| async move {
            let mut result = None;
            tree.send_modify(|tree| {
                if witness == Witness::Keep {
                    // If we're at quota for number of commitments, forget until we're strictly below
                    // quota again, so we can insert something
                    if let Some(max_witnesses) = max_witnesses {
                        let required_forgessions =
                            (1 + tree.witnessed_count()).saturating_sub(max_witnesses);
                        for commitment in
                            random_commitments(&mut *rng.lock(), tree, required_forgessions)
                        {
                            tree.forget(commitment);
                        }
                    }
                }
                // Now actually insert the commitment we wanted to insert
                result = Some(tree.insert(
                    witness,
                    // If no commitment is specified, generate a random one
                    commitment.unwrap_or_else(|| Commitment::random(&mut *rng.lock())),
                ));
            });
            match result.take().unwrap() {
                Ok(position) => Ok(Json(json!({
                    "epoch": position.epoch(),
                    "block": position.block(),
                    "commitment": position.commitment(),
                }))),
                Err(e) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": e.to_string() })),
                )),
            }
        },
    )
}

fn random_commitments<R: Rng>(mut rng: R, tree: &Tree, amount: usize) -> Vec<Commitment> {
    tree.commitments()
        .map(|(c, _)| c)
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, amount)
        .copied()
        .collect()
}

fn forget<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct Forget {
        commitment: Option<Commitment>,
    }

    post(|Query(Forget { commitment }): Query<Forget>| async move {
        let mut result = None;
        tree.send_modify(|tree| {
            if let Some(commitment) = commitment.or_else(|| {
                // If no commitment is specified, forget a random extant one
                random_commitments(&mut *rng.lock(), tree, 1).pop()
            }) {
                result = Some(tree.forget(commitment));
            } else {
                // If no commitment is specified and the tree contains no commitments to
                // forget, return that no commitments were forgotten
                result = Some(false);
            }
        });
        {
            Json(json!(result.take().unwrap()))
        }
    })
}

fn insert_block_root<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct InsertBlockRoot {
        block_root: Option<block::Root>,
    }

    post(
        |Query(InsertBlockRoot { block_root }): Query<InsertBlockRoot>| async move {
            let mut result = None;
            tree.send_modify(|tree| {
                result = Some(tree.insert_block(
                    // If no block root is specified, generate a random one
                    block_root.unwrap_or_else(|| block::Root::random(&mut *rng.lock())),
                ));
            });
            match result.take().unwrap() {
                Ok(block_root) => Ok(block_root.to_string()),
                Err(e) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": e.to_string() })),
                )),
            }
        },
    )
}

fn end_block(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    post(|| async move {
        let mut result = None;
        tree.send_modify(|tree| {
            result = Some(tree.end_block());
        });
        match result.take().unwrap() {
            Ok(block_root) => Ok(Json(block_root.to_string())),
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )),
        }
    })
}

fn insert_epoch_root<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct InsertEpochRoot {
        epoch_root: Option<epoch::Root>,
    }

    post(
        |Query(InsertEpochRoot { epoch_root }): Query<InsertEpochRoot>| async move {
            let mut result = None;
            tree.send_modify(|tree| {
                result = Some(tree.insert_epoch(
                    // If no epoch root is specified, generate a random one
                    epoch_root.unwrap_or_else(|| epoch::Root::random(&mut *rng.lock())),
                ));
            });
            match result.take() {
                Some(Ok(epoch_root)) => Ok(Json(epoch_root.to_string())),
                Some(Err(e)) => Err((StatusCode::BAD_REQUEST, e.to_string())),
                None => Err((StatusCode::INTERNAL_SERVER_ERROR, "".to_string())),
            }
        },
    )
}

fn end_epoch(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    post(|| async move {
        let mut result = None;
        tree.send_modify(|tree| {
            result = Some(tree.end_epoch());
        });
        match result.take() {
            Some(Ok(epoch_root)) => Ok(Json(epoch_root.to_string())),
            Some(Err(e)) => Err((StatusCode::BAD_REQUEST, e.to_string())),
            None => Err((StatusCode::INTERNAL_SERVER_ERROR, "".to_string())),
        }
    })
}
