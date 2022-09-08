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
    /// The maximum number of commitments to permit witnessing before shedding them randomly.
    ///
    /// This is good to set if exposing this server to concurrent users, because it prevents a DoS
    /// attack where someone keeps adding witnesses until the server runs out of memory and/or
    /// clients fall over because they can't handle the size of the tree.
    #[clap(long)]
    max_witnesses: Option<usize>,
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

    let app = live::edit(rng, tree, ext, args.max_witnesses)
        .merge(key_control())
        .layer(TraceLayer::new_for_http());

    let address = ([0, 0, 0, 0], args.port).into();
    help_text(&address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn help_text(address: &std::net::SocketAddr) {
    println!("Serving at http://{address}/view ...");
    println!();
    println!("Keyboard commands available in the browser:");
    println!();
    println!("  - 'n': reset the tree to new");
    println!("  - 'c': insert a random commitment without remembering it");
    println!("  - 'C': insert a random commitment and remember it");
    println!("  - 'b': end the current block");
    println!("  - 'B': insert a random block root");
    println!("  - 'e': end the current epoch");
    println!("  - 'E': insert a random epoch root");
    println!("  - 'f': forget a random commitment");
    println!("  - 'r': evaluate the root of the tree");
    println!();
    println!("Prefix a command key with a number to repeat it, vim-style.");
    println!("For example, '3f' will forget three commitments randomly.");
    println!();
    println!("Mouse over an epoch, block, commitment, or hash to see more info.");
    println!();
    println!("Scroll and drag to zoom and pan.");
    println!();
    println!("Press Ctrl+C in this terminal to stop the server.");
    println!();
    println!("Have fun!");
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
