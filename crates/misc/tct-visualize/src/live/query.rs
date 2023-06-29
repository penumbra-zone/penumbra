use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, MethodRouter},
    Json, Router,
};

use serde_json::json;
use tokio::sync::watch;

use penumbra_tct::{
    structure::{self, Hash},
    StateCommitment, Tree,
};

/// An [`axum`] [`Router`] that serves a `GET` endpoint mirroring the immutable methods of [`Tree`].
///
/// The returned [`watch::Receiver`] issues a change notification whenever interior mutation causes
/// the tree's interior hashes to be evaluated when they previously were not.
pub fn query(tree: watch::Receiver<Tree>) -> (Router, watch::Receiver<()>) {
    let (mark_change, changed) = watch::channel(());
    let mark_change = Arc::new(mark_change);

    (
        Router::new()
            .route("/root", root(tree.clone(), mark_change.clone()))
            .route(
                "/current-block-root",
                current_block_root(tree.clone(), mark_change.clone()),
            )
            .route(
                "/current-epoch-root",
                current_epoch_root(tree.clone(), mark_change.clone()),
            )
            .route("/position", position(tree.clone(), mark_change.clone()))
            .route("/forgotten", forgotten(tree.clone(), mark_change.clone()))
            .route(
                "/witness/:commitment",
                witness(tree.clone(), mark_change.clone()),
            )
            .route(
                "/position-of/:commitment",
                position_of(tree.clone(), mark_change.clone()),
            )
            .route(
                "/witnessed-count",
                witnessed_count(tree.clone(), mark_change.clone()),
            )
            .route("/is-empty", is_empty(tree.clone(), mark_change.clone()))
            .route("/is-full", is_full(tree.clone(), mark_change.clone()))
            .route(
                "/commitments",
                commitments(tree.clone(), mark_change.clone()),
            )
            .route(
                "/commitments-ordered",
                commitments_ordered(tree, mark_change),
            ),
        changed,
    )
}

/// Perform the action on the tree, sending a notification to the watch sender if the tree's
/// frontier has changed
fn marking_change<R>(
    mark_change: Arc<watch::Sender<()>>,
    tree: watch::Receiver<Tree>,
    f: impl Fn(&Tree) -> R,
) -> R {
    let tree = tree.borrow();
    let before = frontier_hashes(&tree);
    let result = f(&tree);
    let after = frontier_hashes(&tree);
    if before != after {
        let _ = mark_change.send(());
    }
    result
}

/// Compute all the frontier hashes of the tree, without forcing them to be evaluated
pub(super) fn frontier_hashes(tree: &Tree) -> Vec<Option<Hash>> {
    fn inner(frontier: &mut Vec<Option<Hash>>, node: structure::Node) {
        frontier.push(node.cached_hash());
        if let Some(rightmost) = node.children().last() {
            inner(frontier, *rightmost);
        }
    }

    let mut frontier = Vec::new();
    inner(&mut frontier, tree.structure());
    frontier
}

fn root(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::root)) })
}

fn current_block_root(
    tree: watch::Receiver<Tree>,
    mark_change: Arc<watch::Sender<()>>,
) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::current_block_root)) })
}

fn current_epoch_root(
    tree: watch::Receiver<Tree>,
    mark_change: Arc<watch::Sender<()>>,
) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::current_epoch_root)) })
}

fn position(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move {
        Json(
            if let Some(position) = marking_change(mark_change, tree, Tree::position) {
                json!({
                    "epoch": position.epoch(),
                    "block": position.block(),
                    "commitment": position.commitment(),
                })
            } else {
                json!(null)
            },
        )
    })
}

fn forgotten(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::forgotten)) })
}

fn witness(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|Path(commitment): Path<StateCommitment>| async move {
        if let Some(witness) = marking_change(mark_change, tree, |tree| tree.witness(commitment)) {
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

fn position_of(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|Path(commitment): Path<StateCommitment>| async move {
        if let Some(position) =
            marking_change(mark_change, tree, |tree| tree.position_of(commitment))
        {
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

fn witnessed_count(
    tree: watch::Receiver<Tree>,
    mark_change: Arc<watch::Sender<()>>,
) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::witnessed_count)) })
}

fn is_empty(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move { Json(marking_change(mark_change, tree, Tree::is_empty)) })
}

fn is_full(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move {
        Json(marking_change(mark_change, tree, |tree| {
            tree.position().is_none()
        }))
    })
}

fn commitments(tree: watch::Receiver<Tree>, mark_change: Arc<watch::Sender<()>>) -> MethodRouter {
    get(|| async move {
        Json(marking_change(mark_change, tree, |tree| {
            tree.commitments_unordered()
                .map(|(commitment, position)| {
                    json!({
                        "commitment": commitment,
                        "position": {
                            "epoch": position.epoch(),
                            "block": position.block(),
                            "commitment": position.commitment()
                        } })
                })
                .collect::<Vec<_>>()
        }))
    })
}

fn commitments_ordered(
    tree: watch::Receiver<Tree>,
    mark_change: Arc<watch::Sender<()>>,
) -> MethodRouter {
    get(|| async move {
        Json(marking_change(mark_change, tree, |tree| {
            tree.commitments()
                .map(|(position, commitment)| {
                    json!({
                        "commitment": commitment,
                        "position": {
                            "epoch": position.epoch(),
                            "block": position.block(),
                            "commitment": position.commitment()
                        } })
                })
                .collect::<Vec<_>>()
        }))
    })
}
