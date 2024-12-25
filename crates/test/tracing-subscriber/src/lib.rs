//! A pretty [`tracing`] subscriber for use in test cases.
//!
//! NB: this subscriber makes use of a test writer, that is compatible with `cargo test`'s output
//! capturing.

use {
    tracing::subscriber::{set_default, DefaultGuard},
    tracing_subscriber::{filter::EnvFilter, fmt},
};

/// Installs a tracing subscriber to log events until the returned guard is dropped.
//  NB: this is marked as "dead code" but it is used by integration tests.
#[allow(dead_code)]
pub fn set_tracing_subscriber() -> DefaultGuard {
    let filter = "info,penumbra_sdk_app=trace,penumbra_sdk_mock_consensus=trace";
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .expect("should have a valid filter directive");
    set_tracing_subscriber_with_env_filter(filter)
}

/// Install a tracing subscriber with a custom filter.
//  NB: this is marked as "dead code" but it is used by integration tests.
#[allow(dead_code)]
pub fn set_tracing_subscriber_with_env_filter(filter: EnvFilter) -> DefaultGuard {
    // Without explicitly disabling the `r1cs` target, the ZK proof implementations
    // will spend an enormous amount of CPU and memory building useless tracing output.
    let filter = filter.add_directive(
        "r1cs=off"
            .parse()
            .expect("rics=off is a valid filter directive"),
    );

    let subscriber = fmt()
        .with_env_filter(filter)
        .pretty()
        .with_test_writer()
        .finish();

    set_default(subscriber)
}
