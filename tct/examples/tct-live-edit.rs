#![recursion_limit = "256"]

use std::sync::Arc;

use axum::Router;
use tokio::sync::watch;
use tower_http::trace::TraceLayer;

use penumbra_tct::{live, Tree};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (control, view) = watch::channel(Tree::new());

    let app = Router::new()
        .merge(live::control(rand::rngs::OsRng, Arc::new(control)))
        .merge(live::query(view.clone()))
        .nest("/view", live::view(view))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
