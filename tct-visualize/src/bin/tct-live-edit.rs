use std::{path::PathBuf, sync::Arc};

use axum::{headers::ContentType, routing::get, Router, TypedHeader};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use include_flate::flate;
use tokio::sync::watch;
use tower_http::trace::TraceLayer;

use penumbra_tct::Tree;
use penumbra_tct_visualize::live::{self, ViewExtensions};

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
    /// The maximum number of times an API call can ask the server to repeat the same operation,
    /// server-side.
    ///
    /// This does not rate-limit repeated requests, but prevents a single request from requiring
    /// more than a certain upper-bound of server-side work.
    #[clap(long)]
    max_repeat: Option<u16>,
    /// The path to a TLS certificate file to use when serving the demo.
    #[clap(long, required_if("tls_key", "is_some"))]
    tls_cert: Option<PathBuf>,
    /// The path to a TLS private key file to use when serving the demo.
    ///
    /// This must be the private key corresponding to the certificate given by `--tls-cert`.
    #[clap(long, required_if("tls_cert", "is_some"))]
    tls_key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let app = live::edit(
        rng,
        tree,
        ext,
        args.max_witnesses.unwrap_or(usize::MAX),
        args.max_repeat.unwrap_or(u16::MAX),
    )
    .merge(key_control())
    .merge(main_help())
    .layer(TraceLayer::new_for_http());
    // TODO: add rate limit layer here

    let address = ([0, 0, 0, 0], args.port).into();
    help_text(&address);

    match (args.tls_cert, args.tls_key) {
        (Some(cert_path), Some(key_path)) => {
            let config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            axum_server::bind_rustls(address, config)
                .serve(app.into_make_service())
                .await
                .unwrap()
        }
        (None, None) => {
            axum::Server::bind(&address)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        _ => unreachable!("both --tls-cert and --tls-key are mandated together"),
    }

    Ok(())
}

fn help_text(address: &std::net::SocketAddr) {
    println!("Serving at http://{address} ...");
}

// Serve the static file "key-control.js", which reads keystrokes and translates them into POST
// requests to the control endpoint:

const KEY_CONTROL_JS_URL: &str = "/scripts/key-control.js";
flate!(static KEY_CONTROL_JS: str from "src/bin/key-control.js");

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

flate!(static HELP_HTML: str from "src/bin/tct-live-edit-help.html");

fn main_help() -> Router {
    Router::new().route(
        "/",
        get(|| async { (TypedHeader(ContentType::html()), HELP_HTML.clone()) }),
    )
}
