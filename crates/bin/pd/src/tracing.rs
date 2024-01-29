//! Utilies for structured logging and diagnostics.
//!
//! Refer to the [`tracing`] crate-level documentation for more information.

use {
    console_subscriber::ConsoleLayer,
    metrics_tracing_context::MetricsLayer,
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter},
};

/// Attempts to set a [global default subscriber] in the current scope.
///
/// # Panics
///
/// This function will panic if this fails. This may mean e.g. that a global default subscriber
/// has already been set. See [`SubscriberInitExt::init()`] for more information.
pub fn init(tokio_console: bool) -> Result<(), anyhow::Error> {
    // The MetricsLayer handles enriching metrics output with labels from tracing spans.
    let metrics_layer = MetricsLayer::new();
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Register the tracing subscribers, conditionally enabling tokio console support
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(metrics_layer);
    if tokio_console {
        // The ConsoleLayer enables collection of data for `tokio-console`.
        // The `spawn` call will panic if AddrInUse, so we only spawn if enabled.
        let console_layer = ConsoleLayer::builder().with_default_env().spawn();
        registry.with(console_layer).init();
    } else {
        registry.init();
    }

    Ok(())
}
