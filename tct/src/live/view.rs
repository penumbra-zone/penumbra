use axum::{
    extract::{OriginalUri, Path, Query},
    headers::ContentType,
    http::StatusCode,
    routing::{get, MethodRouter},
    Json, Router, TypedHeader,
};

use serde_json::json;
use tokio::sync::watch;

use crate::{Forgotten, Position, Tree};

mod resources;
use resources::*;

mod earliest;
use earliest::Earliest;

/// An [`axum`] [`Router`] that serves a live, animated view of a [`Tree`] at the `/` path.
///
/// To include this in a more complex page, nest this `Router` with another one serving the rest of
/// another application, and embed the page it serves as an `<iframe>` in another page.
pub fn view(tree: watch::Receiver<Tree>) -> Router {
    Router::new()
        .route("/", index())
        .route("/scripts/:script", scripts())
        .route("/licenses/:script/LICENSE", licenses())
        .route("/styles/:style", styles())
        .route("/dot", render_dot(tree))
}

/// The index page itself, containing only the animated SVG.
///
/// This templates in the correct absolute URI for each script file, which is necessary because this
/// `Router` could be nested.
fn index() -> MethodRouter {
    get(|OriginalUri(url): OriginalUri| async {
        (TypedHeader(ContentType::html()), resources::index(url))
    })
}

/// Required javascript components for rendering and animation.
fn scripts() -> MethodRouter {
    const JS: [(&str, &str); 1] = [("content-type", "application/javascript")];
    const WASM: [(&str, &str); 1] = [("content-type", "application/wasm")];

    get(|Path(script): Path<String>| async move {
        Ok(match script.as_str() {
            "index.js" => (JS, INDEX_JS.clone()),
            "d3.js" => (JS, D3_JS.clone()),
            "graphviz.js" => (JS, GRAPHVIZ_JS.clone()),
            "graphvizlib.wasm" => (WASM, GRAPHVIZ_WASM.clone()),
            "d3-graphviz.js" => (JS, D3_GRAPHVIZ_JS.clone()),
            _ => return Err(StatusCode::NOT_FOUND),
        })
    })
}

/// License files for the bundled javascript components for rendering and animation.
fn licenses() -> MethodRouter {
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
    })
}

/// Styles for the page.
fn styles() -> MethodRouter {
    get(|Path(style): Path<String>| async move {
        Ok((
            [("content-type", "text/css")],
            match style.as_str() {
                "reset.css" => RESET_CSS.clone(),
                _ => return Err(StatusCode::NOT_FOUND),
            },
        ))
    })
}

/// The graphviz DOT endpoint, which is accessed by the index page's javascript.
fn render_dot(mut tree: watch::Receiver<Tree>) -> MethodRouter {
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

        let graph = String::from_utf8(graph)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Return the DOT graph as a response, with appropriate headers
        Ok::<_, (StatusCode, String)>(Json(json!({
            "position": position,
            "forgotten": tree.forgotten(),
            "graph": graph,
        })))
    })
}
