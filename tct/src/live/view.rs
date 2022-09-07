use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{OriginalUri, Path, Query},
    headers::ContentType,
    http::StatusCode,
    response::{sse, Sse},
    routing::{get, MethodRouter},
    Json, Router, TypedHeader,
};

use serde_json::json;
use tokio::sync::{mpsc, watch};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{Forgotten, Position, Tree};

mod resources;
use resources::*;

mod earliest;
use earliest::Earliest;

/// An [`axum`] [`Router`] that serves a live, animated view of a [`Tree`] at the `/` path.
///
/// To include this in a more complex page, nest this `Router` with another one serving the rest of
/// another application, and embed the page it serves as an `<iframe>` in another page.
pub fn view(tree: watch::Receiver<Tree>, ext: ViewExtensions) -> Router {
    Router::new()
        .route("/", index(ext))
        .route("/scripts/:script", scripts())
        .route("/licenses/:script/LICENSE", licenses())
        .route("/styles/:style", styles())
        .route("/changes", changes(tree.clone()))
        .route("/dot", render_dot(tree))
}

/// Extra HTML fragments to insert into the HTML page that renders the view.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ViewExtensions {
    /// Extra HTML to insert into the `<head>` of the page after the existing script and style tags.
    pub head: String,
    /// Extra HTML to insert into the `<body>` of the page before the graph.
    pub before: String,
    /// Extra HTML to insert into the `<body>` of the page after the graph.
    pub after: String,
}

/// The index page itself, containing only the animated SVG.
///
/// This templates in the correct absolute URI for each script file, which is necessary because this
/// `Router` could be nested.
fn index(ext: ViewExtensions) -> MethodRouter {
    let ext = Arc::new(ext);
    get(move |OriginalUri(url): OriginalUri| {
        let ext = ext.clone();
        async move {
            (
                TypedHeader(ContentType::html()),
                resources::index(url, &ext.head, &ext.before, &ext.after),
            )
        }
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
                "style.css" => STYLE_CSS.clone(),
                _ => return Err(StatusCode::NOT_FOUND),
            },
        ))
    })
}

/// SSE endpoint that sends an event with the tree's position and forgotten count every time the
/// tree is changed.
///
/// The returned data will stay the same if the tree experiences interior mutation.
fn changes(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(move || async move {
        // Clone the watch receiver so we don't steal other users' updates
        let mut tree = tree.clone();

        let (tx, rx) = mpsc::channel(1);

        // Forward all changes to the tree as events
        tokio::spawn(async move {
            loop {
                // Wait for something to change
                if (tree.changed().await).is_err() {
                    break;
                }

                // Form one or two events about it
                let changed = {
                    let tree = tree.borrow();
                    let forgotten = tree.forgotten();
                    let position = if let Some(position) = tree.position() {
                        json!({
                            "epoch": position.epoch(),
                            "block": position.block(),
                            "commitment": position.commitment(),
                        })
                    } else {
                        json!(null)
                    };

                    // Always report the current position and forgotten count
                    sse::Event::default()
                        .event("changed")
                        .json_data(json!({ "position": position, "forgotten": forgotten }))
                        .unwrap()
                };

                if (tx.send(changed).await).is_err() {
                    break;
                }
            }
        });

        Sse::new(ReceiverStream::new(rx).map(Ok::<_, Infallible>))
    })
}

/// The graphviz DOT endpoint, which is accessed by the index page's javascript.
fn render_dot(tree: watch::Receiver<Tree>) -> MethodRouter {
    get(move |Query(earliest): Query<Earliest>| async move {
        // Clone the watch receiver so we don't steal other users' updates
        let mut tree = tree.clone();

        // Wait for the tree to reach the requested position and forgotten index
        loop {
            {
                // Extra scope necessary to satisfy borrow checker
                let current = tree.borrow();
                if earliest.not_too_late_for(&current) {
                    tracing::debug!(
                        forgotten = ?current.forgotten(),
                        position = ?current.position(),
                        earliest = ?earliest,
                        "delivering desired tree version"
                    );
                    break;
                } else {
                    tracing::debug!(
                        forgotten = ?current.forgotten(),
                        position = ?current.position(),
                        earliest = ?earliest,
                        "waiting for later tree version ..."
                    );
                }
            }
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
