use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    body::StreamBody,
    extract::{OriginalUri, Path, Query},
    headers::ContentType,
    http::StatusCode,
    response::{
        sse::{self, KeepAlive},
        Sse,
    },
    routing::{get, MethodRouter},
    Router, TypedHeader,
};

use bytes::Bytes;
use futures::stream;
use serde_json::json;
use tokio::sync::{mpsc, watch};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{Forgotten, Position, Tree};

mod resources;
use resources::*;

mod earliest;
use earliest::DotQuery;

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
        .route("/extra-changes", extra_changes(tree.clone()))
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
/// tree is changed and its position and forgotten count remain the same, or they go backwards.
///
/// This allows the listener to detect interior mutation and resets.
fn extra_changes(mut tree: watch::Receiver<Tree>) -> MethodRouter {
    get(move || async move {
        let (tx, rx) = mpsc::channel(1);

        // Forward all changes to the tree as events
        tokio::spawn(async move {
            let (mut position, mut forgotten) = {
                let tree = tree.borrow();
                (tree.position(), tree.forgotten())
            };

            loop {
                // Wait for something to change
                if (tree.changed().await).is_err() {
                    break;
                }

                // Form an event about it
                let changed = {
                    let tree = tree.borrow();
                    let new_position = tree.position();
                    let new_forgotten = tree.forgotten();

                    // If the position or forgotten has changed, don't emit an event, because the
                    // subscriber only cares about changes they can't detect with long-polling
                    let strictly_forward =
                        // Position and forgotten both don't go backward
                        (new_position >= position && new_forgotten >= forgotten)
                        // At least one of them goes forward
                        && (new_position > position || new_forgotten > forgotten);

                    // Keep track of the new position and forgotten for the next loop around
                    position = new_position;
                    forgotten = new_forgotten;

                    // If we moved forward, don't emit an event
                    if strictly_forward {
                        continue;
                    }

                    let position = if let Some(position) = new_position {
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
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(5)))
    })
}

/// Render a snapshot of the tree in the [`watch::Receiver`] as a DOT graph, then escaped as a JSON string.
fn render_from_watch(tree: &watch::Receiver<Tree>) -> (Tree, Bytes) {
    let tree = tree.borrow().clone();
    let mut dot = Vec::new();
    tree.render_dot(&mut dot).unwrap();
    let dot_json_string = serde_json::to_vec(&json!(String::from_utf8(dot).unwrap())).unwrap();
    (tree, dot_json_string.into())
}

/// Spawns a render loop which is triggered by new requests.
///
/// Returns a closure which triggers a new render and hands back a ticket number, and a watch
/// receiver which can be used to monitor for updates to the latest rendered version.
fn spawn_render_worker(
    tree: &watch::Receiver<Tree>,
) -> (
    impl Fn() -> u64 + Send + Sync + 'static,
    watch::Receiver<(u64, Tree, Bytes)>,
) {
    let (request_render, mut receive_request) = watch::channel(0);
    let (submit_render, receive_render) = watch::channel({
        let (tree, dot_json_string) = render_from_watch(tree);
        (0, tree, dot_json_string)
    });

    tokio::spawn({
        let tree = tree.clone();
        async move {
            while let Ok(()) = receive_request.changed().await {
                let i = *receive_request.borrow();
                let (tree, dot_json_string) = render_from_watch(&tree);
                if submit_render.send((i, tree, dot_json_string)).is_err() {
                    break;
                }
            }
        }
    });

    // This closure requests a new render
    let request_render = move || {
        let mut ticket = None;
        request_render.send_modify(|i| {
            // Increment the render ticket counter
            *i += 1;
            // Report the ticket number associated with this request
            ticket = Some(*i);
        });
        ticket.unwrap()
    };

    (request_render, receive_render)
}

/// The graphviz DOT endpoint, which is accessed by the index page's javascript.
fn render_dot(mut tree: watch::Receiver<Tree>) -> MethodRouter {
    let (request_render, mut receive_render) = spawn_render_worker(&tree);
    let request_render = Arc::new(request_render);

    get(move |Query(query): Query<DotQuery>| async move {
        // Wait for the tree to reach the requested position and forgotten index
        loop {
            {
                // Extra scope necessary to satisfy borrow checker
                let current = tree.borrow();
                if query.not_too_late_for(&current) {
                    break;
                }
            }
            tree.changed().await.unwrap();
        }

        // If the graph is not requested, don't render it, just return the other data
        if !query.graph {
            return Ok::<_, (StatusCode, String)>((
                TypedHeader(ContentType::json()),
                StreamBody::new(
                    stream::iter(vec![json!({
                        "position": tree.borrow().position(),
                        "forgotten": tree.borrow().forgotten(),
                    })
                    .to_string()
                    .into()])
                    .map(Ok),
                ),
            ));
        }

        // Now that the tree is the right version, loop until a suitable render is provided
        let (position, forgotten, rendered) = loop {
            // Subscribe to change notifications before sending the render request, to make sure we
            // don't miss any updates
            let changed = receive_render.changed();
            // Each render request is associated with a ticket number, the latest of which is sent
            // back when a render is completed; this allows us to know whether the render is current
            // as of when we made the request
            let ticket = request_render();
            changed.await.unwrap();

            {
                // Once there's an updated render, it might still not be the right version for us,
                // because it might have been triggered by someone else before we asked, so check to
                // make sure that the ticket number associated with this render request was from
                // us, or someone who asked afterwards that
                let rendered = receive_render.borrow();
                // We use two different criteria for the tree being the right version:
                //
                // - If the request was a long-polling request, then any up-to-date version of the
                //   tree will do, even if the precise ticket hasn't been fulfilled yet, since there
                //   will be another update coming again as soon as the long poll completes.
                //
                // - If the request was a short-polling request, then we need to wait for the exact
                //   ticket to be fulfilled, because it's not necessarily the case that there will
                //   be another repeated request, so we need to serve the absolutely most current
                //   version of the tree as of the request.
                //
                // This precise criterion means that the client will receive responses to
                // long-polling as soon as something valid is available, even if that valid thing
                // ends up being a little stale, while requests for something precisely up to date
                // will wait for the render task to finish past that specific request.
                if (query.next && query.not_too_late_for(&rendered.1))
                    || (!query.next && ticket <= rendered.0)
                {
                    break (
                        rendered.1.position(),
                        rendered.1.forgotten(),
                        rendered.2.clone(),
                    );
                }
            }
        };

        let position = if let Some(position) = position {
            json!({
                "epoch": position.epoch(),
                "block": position.block(),
                "commitment": position.commitment(),
            })
        } else {
            json!(null)
        };

        // Return the DOT graph as a response, with appropriate headers
        Ok::<_, (StatusCode, String)>((
            TypedHeader(ContentType::json()),
            // Manually construct a streaming response to avoid allocating a copy of the large
            // rendered bytes: the graph is already rendered as a JSON-escaped string, so we include
            // it literally in this output
            StreamBody::new(
                stream::iter(vec![
                    "{".into(),
                    "\"position\":".into(),
                    serde_json::to_vec(&position).unwrap().into(),
                    ",\"forgotten\":".into(),
                    serde_json::to_vec(&forgotten).unwrap().into(),
                    ",\"graph\":".into(),
                    rendered,
                    "}".into(),
                ])
                .map(Ok::<Bytes, Infallible>),
            ),
        ))
    })
}
