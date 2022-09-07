#![recursion_limit = "256"]

use std::sync::Arc;

use clap::Parser;
use tokio::sync::watch;
use tower_http::trace::TraceLayer;

use penumbra_tct::{live, Tree};

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

    let tree = Arc::new(watch::channel(Tree::new()).0);
    let rng = rand::rngs::OsRng;

    let app = live::edit(rng, tree).layer(TraceLayer::new_for_http());

    axum::Server::bind(&([0, 0, 0, 0], args.port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
