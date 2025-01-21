//! A web service to view the live state of the TCT.

use std::sync::Arc;

use axum::Router;

mod view;
use rand::Rng;
use tokio::sync::watch;
pub use view::{view, ViewExtensions};

mod query;
pub use query::query;

mod control;
pub use control::control;

use penumbra_sdk_tct::Tree;

/// Combine the [`control`], [`query`], and [`view`] endpoints into a single [`Router`].
///
/// This additionally hooks up the change notification from implicit interior mutation to the tree
/// during queries, so that the live view reflects these changes instantly.
pub fn edit<R: Rng + Send + 'static>(
    rng: R,
    tree: Arc<watch::Sender<Tree>>,
    ext: ViewExtensions,
    max_witnesses: usize,
    max_repeat: u16,
) -> Router {
    // The three endpoints
    let control = control(rng, tree.clone(), max_witnesses, max_repeat);
    let (query, mut changed) = query(tree.subscribe());
    let view = view(tree.subscribe(), ext);

    // Background task to notify listeners to the tree that interior mutation has been triggered by
    // a query issued by the query endpoint
    tokio::spawn(async move {
        while let Ok(()) = changed.changed().await {
            tree.send_modify(|_| {});
        }
    });

    Router::new()
        .merge(control)
        .merge(query)
        .nest("/view", view)
}
