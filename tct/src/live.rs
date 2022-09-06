//! A web service to view the live state of the TCT.

use axum::{
    extract::{Path, Query},
    headers::ContentType,
    http::StatusCode,
    routing::get,
    Json, Router, TypedHeader,
};

use serde_json::json;
use tokio::sync::watch;

use crate::{Commitment, Forgotten, Position, Tree};

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
            let tree = tree.clone();
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
