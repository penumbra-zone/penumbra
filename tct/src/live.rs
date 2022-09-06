//! A web service to view the live state of the TCT.

use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    headers::ContentType,
    http::StatusCode,
    routing::{get, post},
    Json, Router, TypedHeader,
};

use parking_lot::Mutex;
use rand::{seq::SliceRandom, Rng};
use serde_json::json;
use tokio::sync::watch;

use crate::{
    builder::{block, epoch},
    Commitment, Forgotten, Position, Tree, Witness,
};

mod earliest;
use earliest::Earliest;

/// An [`axum`] [`Router`] that serves a live, animated view of a [`Tree`] at the `/` path.
///
/// To include this in a more complex page, nest this `Router` with another one serving the rest of
/// another application, and embed the page it serves as an `<iframe>` in another page.
pub async fn view(mut tree: watch::Receiver<Tree>) -> Router {
    Router::new()
        // The index page itself, containing only the animated SVG
        .route(
            "/",
            get(|| async { (TypedHeader(ContentType::html()), INDEX.clone()) }),
        )
        // Required javascript components for rendering and animation
        .route(
            "/scripts/:script.js",
            get(|Path(script): Path<String>| async move {
                Ok((
                    [("content-type", "application/javascript")], // There is no application/javascript in `TypedHeader`
                    match script.as_str() {
                        "d3" => D3_JS.clone(),
                        "graphviz" => GRAPHVIZ_JS.clone(),
                        "d3-graphviz" => D3_GRAPHVIZ_JS.clone(),
                        _ => return Err(StatusCode::NOT_FOUND),
                    },
                ))
            }),
        )
        // Required javascript components for rendering and animation
        .route(
            "/scripts/:script/LICENSE",
            get(|Path(script): Path<String>| async move {
                Ok((
                    TypedHeader(ContentType::text_utf8()),
                    match script.as_str() {
                        "d3.js" => D3_JS_LICENSE.clone(),
                        "graphviz.js" => GRAPHVIZ_JS_LICENSE.clone(),
                        "d3-graphviz.js" => D3_GRAPHVIZ_JS_LICENSE.clone(),
                        _ => return Err(StatusCode::NOT_FOUND),
                    },
                ))
            }),
        )
        // The graphviz DOT endpoint, which is accessed by the index page's javascript
        .route(
            "/dot",
            get(move |earliest: Query<Earliest>| async move {
                // Wait for the tree to reach the requested position and forgotten index
                while !earliest.earlier_than(&tree.borrow()) {
                    tree.changed().await.unwrap();
                }

                // This clone means we don't hold a read lock on the tree for long, because cloning
                // the tree is much faster than generating the DOT representation
                let tree = tree.borrow().clone();

                // Render the tree as a DOT graph
                let mut graph = Vec::new();
                tree.render_dot(&mut graph)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                let position = if let Some(position) = tree.position() {
                    json!({
                        "epoch": position.epoch(),
                        "block": position.block(),
                        "commitment": position.commitment(),
                    })
                } else {
                    json!(null)
                };

                // Return the DOT graph as a response, with appropriate headers
                Ok::<_, (StatusCode, String)>(Json(json!({
                    "position": position,
                    "forgotten": tree.forgotten(),
                    "graph": graph,
                })))
            }),
        )
}

/// An [`axum`] [`Router`] that serves a `GET` endpoint mirroring the immutable methods of [`Tree`].
pub async fn query(tree: watch::Receiver<Tree>) -> Router {
    Router::new()
        .route("/root", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().root().to_string()) })
        })
        .route("/current-block-root", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().current_block_root().to_string()) })
        })
        .route("/current-epoch-root", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().current_epoch_root().to_string()) })
        })
        .route("/position", {
            let tree = tree.clone();
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
        })
        .route("/forgotten", {
            let tree = tree.clone();
            get(|| async move { Json(u64::from(tree.borrow().forgotten())) })
        })
        .route("/witness/:commitment", {
            let tree = tree.clone();
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
        })
        .route("/position-of/:commitment", {
            let tree = tree.clone();
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
        })
        .route("/witnessed-count", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().witnessed_count()) })
        })
        .route("/is-empty", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().is_empty()) })
        })
        .route("/is-full", {
            let tree = tree.clone();
            get(|| async move { Json(tree.borrow().position().is_none()) })
        })
        .route("/commitments", {
            let tree = tree.clone();
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
        })
        .route("/commitments-ordered", {
            let tree = tree;
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
        })
}

/// An [`axum`] [`Router`] that serves a `POST` endpoint for updating the [`Tree`].
///
/// Queries taking arguments pass arguments via URL parameters. Results are returned in JSON format,
/// with [`StatusCode::BAD_REQUEST`] being returned if the operation failed (i.e. an `Err` variant
/// was returned).
pub async fn control<R: Rng + Send + 'static>(rng: R, tree: Arc<watch::Sender<Tree>>) -> Router {
    let rng = Arc::new(Mutex::new(rng));

    Router::new()
        .route("/new", {
            let tree = tree.clone();
            post(move || async move {
                tree.send_modify(|tree| {
                    *tree = Tree::new();
                })
            })
        })
        .route("/insert", {
            #[derive(Deserialize)]
            struct Insert {
                witness: Witness,
                commitment: Option<Commitment>,
            }

            let tree = tree.clone();
            let rng = rng.clone();
            post(
                move |Query(Insert {
                          witness,
                          commitment,
                      }): Query<Insert>| async move {
                    let mut result = None;
                    tree.send_modify(|tree| {
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
        })
        .route("/forget", {
            #[derive(Deserialize)]
            struct Forget {
                commitment: Option<Commitment>,
            }

            let tree = tree.clone();
            let rng = rng.clone();
            post(
                move |Query(Forget { commitment }): Query<Forget>| async move {
                    let mut result = None;
                    tree.send_modify(|tree| {
                        if let Some(commitment) = commitment.or_else(|| {
                            // If no commitment is specified, forget a random one that is present in
                            // the tree
                            tree.commitments()
                                .map(|(c, _)| c)
                                .collect::<Vec<_>>()
                                .choose(&mut *rng.lock())
                                .copied()
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
                },
            )
        })
        .route("/insert-block-root", {
            #[derive(Deserialize)]
            struct InsertBlockRoot {
                block_root: Option<block::Root>,
            }

            let tree = tree.clone();
            let rng = rng.clone();
            post(
                move |Query(InsertBlockRoot { block_root }): Query<InsertBlockRoot>| async move {
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
        })
        .route("/end-block", {
            let tree = tree.clone();
            post(move || async move {
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
        })
        .route("/insert-epoch-root", {
            #[derive(Deserialize)]
            struct InsertEpochRoot {
                epoch_root: Option<epoch::Root>,
            }

            let tree = tree.clone();
            post(
                move |Query(InsertEpochRoot { epoch_root }): Query<InsertEpochRoot>| async move {
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
        })
        .route("/end-epoch", {
            post(move || async move {
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
        })
}

// This is a modified variant of the `flate` macro from the `include_flate` crate, which makes a
// `Bytes` value, so that we can avoid expensive cloning of large strings.
macro_rules! flate_bytes {
    ($(#[$meta:meta])*
        $(pub $(($($vis:tt)+))?)? static $name:ident from $path:literal) => {
        ::include_flate::lazy_static! {
            $(#[$meta])*
            $(pub $(($($vis)+))?)? static ref $name: ::bytes::Bytes = ::include_flate::decode_string(::include_flate::codegen::deflate_utf8_file!($path)).into();
        }
    };
}

// Embed compressed index page
flate_bytes!(static INDEX from "src/live/index.html");

// Embed compressed source for the relevant javascript libraries
flate_bytes!(static D3_JS from "node_modules/d3/dist/d3.min.js");
flate_bytes!(static GRAPHVIZ_JS from "node_modules/@hpcc-js/wasm/dist/index.min.js");
flate_bytes!(static D3_GRAPHVIZ_JS from "node_modules/d3-graphviz/build/d3-graphviz.js");

// Embed compressed license files for the relevant javascript libraries
flate_bytes!(static D3_JS_LICENSE from "node_modules/d3/LICENSE");
flate_bytes!(static GRAPHVIZ_JS_LICENSE from "node_modules/@hpcc-js/wasm/LICENSE");
flate_bytes!(static D3_GRAPHVIZ_JS_LICENSE from "node_modules/d3-graphviz/LICENSE");
