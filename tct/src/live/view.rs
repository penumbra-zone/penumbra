use axum::{
    extract::{Path, Query},
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
pub async fn view(tree: watch::Receiver<Tree>) -> Router {
    Router::new()
        .route("/", index())
        .route("/scripts/:script.js", scripts())
        .route("/scripts/:script/LICENSE", licenses())
        .route("/dot", render_dot(tree))
}

/// The index page itself, containing only the animated SVG.
fn index() -> MethodRouter {
    get(|| async { (TypedHeader(ContentType::html()), INDEX.clone()) })
}

/// Required javascript components for rendering and animation.
fn scripts() -> MethodRouter {
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

        // Return the DOT graph as a response, with appropriate headers
        Ok::<_, (StatusCode, String)>(Json(json!({
            "position": position,
            "forgotten": tree.forgotten(),
            "graph": graph,
        })))
    })
}
