# Metrics

Metrics are an important part of observability, allowing us to understand what
the Penumbra software is doing.

## Viewing Metrics

TODO: add details on how to use Grafana:

- [ ] link to <https://grafana.testnet.penumbra.zone> for dashboard on current testnet;
- [ ] instructions on how to run Grafana + Prometheus for local dev setup (ideally this could work without requiring that `pd` itself is Dockerized, since local development is often more convenient outside of docker);
- [x] instructions on how to commit dashboards back to the repo.

## Adding Metrics

We use a common structure for organizing metrics code throughout the `penumbra`
workspace.  Each crate that uses metrics has a top-level `metrics` module, which
is private to the crate.  That module contains:

- a re-export of the entire metrics crate: `pub use metrics::*;`
- `&'static str` constants for every metrics key used by the crate;
- a `pub fn register_metrics()` that registers and describes all of the metrics used by the crate;

Finally, the `register_metrics` function is publicly re-exported from the crate root.

The only part of this structure visible outside the crate is the
`register_metrics` function in the crate root, allowing users of the library to
register and describe the metrics it uses on startup.

Internally to the crate, all metrics keys are in one place, rather than being
scattered across the codebase, so it's easy to see what metrics there are.
Because the `metrics` _module_ re-exports the contents of the `metrics` _crate_,
doing `use crate::metrics;` is effectively a way to monkey-patch the
crate-specific constants into the `metrics` crate, allowing code like:

```rust
metrics::increment_counter!(
    metrics::MEMPOOL_CHECKTX_TOTAL,
    "kind" => "new",
    "code" => "1"
);
```

The metrics keys themselves should:

- follow the [Prometheus metrics naming guidelines](https://prometheus.io/docs/practices/naming/)
- have an initial prefix of the form `penumbra_CRATE`, e.g., `penumbra_stake`, `penumbra_pd`, etc;
- have some following module prefix that makes sense relative to the other metrics in the crate.

For instance:

```rust
pub const MEMPOOL_CHECKTX_TOTAL: &str = "penumbra_pd_mempool_checktx_total";
```

## Backing up Grafana

After being changed, Grafana dashboards should be backed up to the repository for posterity and redeployment.

Grafana has an [import/export](https://grafana.com/docs/grafana/latest/dashboards/export-import/) feature that
we use for maintaining our dashboards.

1. Export the dashboard as JSON with the default settings
2. Rename the JSON file and copy into the repo (`config/grafana/dashboards/`)
3. Use the import function in the UI to update all deployments
