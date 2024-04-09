use {
    tracing::subscriber::{set_default, DefaultGuard},
    tracing_subscriber::{filter::EnvFilter, fmt},
};

/// Installs a tracing subscriber to log events until the returned guard is dropped.
//  NB: this is marked as "dead code" but it is used by integration tests.
#[allow(dead_code)]
pub fn set_tracing_subscriber() -> DefaultGuard {
    let filter = "info,penumbra_app=trace,penumbra_mock_consensus=trace";
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .expect("should have a valid filter directive")
        // Without explicitly disabling the `r1cs` target, the ZK proof implementations
        // will spend an enormous amount of CPU and memory building useless tracing output.
        .add_directive(
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
