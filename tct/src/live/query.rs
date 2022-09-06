use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, MethodRouter},
    Json, Router,
};

use serde_json::json;
use tokio::sync::watch;

use crate::{Commitment, Tree};

/// An [`axum`] [`Router`] that serves a `GET` endpoint mirroring the immutable methods of [`Tree`].
pub fn query(tree: watch::Receiver<Tree>) -> Router {
    Router::new()
        .route("/root", root(tree.clone()))
        .route("/current-block-root", current_block_root(tree.clone()))
        .route("/current-epoch-root", current_epoch_root(tree.clone()))
        .route("/position", position(tree.clone()))
        .route("/forgotten", forgotten(tree.clone()))
        .route("/witness/:commitment", witness(tree.clone()))
        .route("/position-of/:commitment", position_of(tree.clone()))
        .route("/witnessed-count", witnessed_count(tree.clone()))
        .route("/is-empty", is_empty(tree.clone()))
        .route("/is-full", is_full(tree.clone()))
        .route("/commitments", commitments(tree.clone()))
        .route("/commitments-ordered", commitments_ordered(tree))
}

fn root(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().root().to_string()) })
}

fn current_block_root(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().current_block_root().to_string()) })
}

fn current_epoch_root(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().current_epoch_root().to_string()) })
}

fn position(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move {
        Json(if let Some(position) = tree.borrow().position() {
            json!({
                "epoch": position.epoch(),
                "block": position.block(),
                "commitment": position.commitment(),
            })
        } else {
            json!(null)
        })
    })
}

fn forgotten(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(u64::from(tree.borrow().forgotten())) })
}

fn witness(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|Path(commitment): Path<Commitment>| async move {
        if let Some(witness) = tree.borrow().witness(commitment) {
            Ok(Json(json!({
                "commitment": witness.commitment(),
                "position": {
                    "epoch": witness.position().epoch(),
                    "block": witness.position().block(),
                    "commitment": witness.position().commitment(),
                },
                "auth_path": witness.auth_path(),
            })))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    })
}

fn position_of(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|Path(commitment): Path<Commitment>| async move {
        if let Some(position) = tree.borrow().position_of(commitment) {
            Ok(Json(json!({
                "epoch": position.epoch(),
                "block": position.block(),
                "commitment": position.commitment(),
            })))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    })
}

fn witnessed_count(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().witnessed_count()) })
}

fn is_empty(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().is_empty()) })
}

fn is_full(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move { Json(tree.borrow().position().is_none()) })
}

fn commitments(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move {
        Json(
            tree.borrow()
                .commitments()
                .map(|(commitment, position)| {
                    json!({
                        "commitment": commitment,
                        "position": {
                            "epoch": position.epoch(),
                            "block": position.block(),
                            "commitment": position.commitment()
                        } })
                })
                .collect::<Vec<_>>(),
        )
    })
}

fn commitments_ordered(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(|| async move {
        Json(
            tree.borrow()
                .commitments_ordered()
                .map(|(position, commitment)| {
                    json!({
                        "commitment": commitment,
                        "position": {
                            "epoch": position.epoch(),
                            "block": position.block(),
                            "commitment": position.commitment()
                        } })
                })
                .collect::<Vec<_>>(),
        )
    })
}
