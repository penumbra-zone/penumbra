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
use tokio::{sync::watch, task::spawn_blocking};

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

fn marking_change<R>(tree: Arc<watch::Sender<Tree>>, f: impl Fn(&mut Tree) -> R) -> R {
    let mut result = None;
    tree.send_modify(|tree| {
        // TODO: make this `send_if_modified` to reduce watch channel churn (right now that doesn't
        // work because someone fails to get an update if this is what's done)
        // let forgotten_before = tree.forgotten();
        // let position_before = tree.position();
        // let frontier_before = super::query::frontier_hashes(tree);
        result = Some(f(tree));
        // // Only produce a change notification if the tree has been changed
        // forgotten_before != tree.forgotten()
        //     || position_before != tree.position()
        //     || frontier_before != super::query::frontier_hashes(tree)
    });
    result.unwrap()
}

fn new(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    post(|| async move {
        marking_change(tree, |tree| {
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
        #[serde(default = "one")]
        repeat: u16,
    }

    post(
        move |Query(Insert {
                  witness,
                  commitment,
                  repeat,
              }): Query<Insert>| async move {
            spawn_blocking(move || {
                let mut result = None;

                for _ in 0..repeat {
                    result = Some(marking_change(tree.clone(), |tree| {
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
                        tree.insert(
                            witness,
                            // If no commitment is specified, generate a random one
                            commitment.unwrap_or_else(|| Commitment::random(&mut *rng.lock())),
                        )
                    }));
                }
                match result {
                    Some(Ok(position)) => Ok(Json(json!({
                        "epoch": position.epoch(),
                        "block": position.block(),
                        "commitment": position.commitment(),
                    }))),
                    Some(Err(e)) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": e.to_string() })),
                    )),
                    None => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": "repeat must be greater than zero" })),
                    )),
                }
            })
            .await
            .unwrap()
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
        #[serde(default = "one")]
        repeat: u16,
    }

    post(
        |Query(Forget { commitment, repeat }): Query<Forget>| async move {
            spawn_blocking(move || {
                let mut result = None;
                for _ in 0..repeat {
                    result = Some(marking_change(tree.clone(), |tree| {
                        if let Some(commitment) = commitment.or_else(|| {
                            // If no commitment is specified, forget a random extant one
                            random_commitments(&mut *rng.lock(), tree, 1).pop()
                        }) {
                            tree.forget(commitment)
                        } else {
                            // If no commitment is specified and the tree contains no commitments to
                            // forget, return that no commitments were forgotten
                            false
                        }
                    }));
                }

                match result {
                    Some(result) => Ok(Json(json!(result))),
                    None => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": "repeat must be greater than zero" })),
                    )),
                }
            })
            .await
            .unwrap()
        },
    )
}

fn insert_block_root<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct InsertBlockRoot {
        block_root: Option<block::Root>,
        #[serde(default = "one")]
        repeat: u16,
    }

    post(
        |Query(InsertBlockRoot { block_root, repeat }): Query<InsertBlockRoot>| async move {
            spawn_blocking(move || {
                let mut result = None;
                for _ in 0..repeat {
                    result = Some(marking_change(tree.clone(), |tree| {
                        tree.insert_block(
                            // If no block root is specified, generate a random one
                            block_root.unwrap_or_else(|| block::Root::random(&mut *rng.lock())),
                        )
                    }));
                }
                match result {
                    Some(Ok(block_root)) => Ok(block_root.to_string()),
                    Some(Err(e)) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": e.to_string() })),
                    )),
                    None => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": "repeat must be greater than zero" })),
                    )),
                }
            })
            .await
            .unwrap()
        },
    )
}

fn end_block(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    #[derive(Deserialize)]
    struct EndBlock {
        #[serde(default = "one")]
        repeat: u16,
    }

    post(|Query(EndBlock { repeat }): Query<EndBlock>| async move {
        spawn_blocking(move || {
            let mut result = None;
            for _ in 0..repeat {
                result = Some(marking_change(tree.clone(), |tree| tree.end_block()));
            }
            match result {
                Some(Ok(block_root)) => Ok(Json(block_root.to_string())),
                Some(Err(e)) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": e.to_string() })),
                )),
                None => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "repeat must be greater than zero" })),
                )),
            }
        })
        .await
        .unwrap()
    })
}

fn insert_epoch_root<R: Rng + Send + 'static>(
    rng: Arc<Mutex<R>>,
    tree: Arc<watch::Sender<Tree>>,
) -> MethodRouter {
    #[derive(Deserialize)]
    struct InsertEpochRoot {
        epoch_root: Option<epoch::Root>,
        #[serde(default = "one")]
        repeat: u16,
    }

    post(
        |Query(InsertEpochRoot { epoch_root, repeat }): Query<InsertEpochRoot>| async move {
            spawn_blocking(move || {
                let mut result = None;
                for _ in 0..repeat {
                    result = Some(marking_change(tree.clone(), |tree| {
                        tree.insert_epoch(
                            // If no epoch root is specified, generate a random one
                            epoch_root.unwrap_or_else(|| epoch::Root::random(&mut *rng.lock())),
                        )
                    }));
                }
                match result {
                    Some(Ok(epoch_root)) => Ok(Json(epoch_root.to_string())),
                    Some(Err(e)) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": e.to_string() })),
                    )),
                    None => Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": "repeat must be greater than zero" })),
                    )),
                }
            })
            .await
            .unwrap()
        },
    )
}

fn end_epoch(tree: Arc<watch::Sender<Tree>>) -> MethodRouter {
    #[derive(Deserialize)]
    struct EndEpoch {
        #[serde(default = "one")]
        repeat: u16,
    }

    post(|Query(EndEpoch { repeat }): Query<EndEpoch>| async move {
        spawn_blocking(move || {
            let mut result = None;
            for _ in 0..repeat {
                result = Some(marking_change(tree.clone(), |tree| tree.end_epoch()));
            }
            match result {
                Some(Ok(epoch_root)) => Ok(Json(epoch_root.to_string())),
                Some(Err(e)) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": e.to_string()})),
                )),
                None => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "repeat must be greater than zero" })),
                )),
            }
        })
        .await
        .unwrap()
    })
}

fn one() -> u16 {
    1
}
