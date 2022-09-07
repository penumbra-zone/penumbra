#![recursion_limit = "256"]

use std::sync::Arc;

use axum::{routing::get, Router};
use clap::Parser;
use include_flate::flate;
use tokio::sync::watch;
use tower_http::trace::TraceLayer;

use penumbra_tct::{
    live::{self, ViewExtensions},
    Tree,
};

/// Visualize the structure of the Tiered Commitment Tree.
#[derive(Parser, Debug)]
struct Args {
    /// The port on which to serve the visualization and control API.
    #[clap(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let rng = rand::rngs::OsRng;

    // Make a new tree and wrap it so its shareable and watchable
    let tree = Arc::new(watch::channel(Tree::new()).0);

    // Add a script link to the key control javascript file, which will be served by an added route
    let ext = ViewExtensions {
        after: format!("<script src=\"{KEY_CONTROL_JS_URL}\"></script>"),
        ..Default::default()
    };

    let app = live::edit(rng, tree, ext)
        .merge(key_control())
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&([0, 0, 0, 0], args.port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Serve the static file "key-control.js", which reads keystrokes and translates them into POST
// requests to the control endpoint:

const KEY_CONTROL_JS_URL: &str = "/scripts/key-control.js";
flate!(static KEY_CONTROL_JS: str from "examples/key-control.js");

fn key_control() -> Router {
    Router::new().route(
        KEY_CONTROL_JS_URL,
        get(|| async {
            (
                [("content-type", "application/javascript")],
                KEY_CONTROL_JS.clone(),
            )
        }),
    )
}
