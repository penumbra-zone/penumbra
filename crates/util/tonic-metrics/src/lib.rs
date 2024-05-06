//! Metrics module for emitting Prometheus metrics, specifically
//! for gRPC services created by [tonic].
#![allow(dead_code, unused_imports)]
use pin_project::pin_project;
// use std::collections::BTreeMap;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
// use tower::Layer;
// use tracing_subscriber::layer::SubscriberExt as _;
use tower::Service;
use tracing::instrument;

/// Wrapper that instruments a tower::Service with gRPC-specific metrics.
#[derive(Debug, Clone)]
pub struct TonicMetricsService<S> {
    /// The wrapped service
    service: S,

    // TODO: Read in the proto descriptors at time of construction,
    // and populate `metrics_keys` from that data. We want to ensure that allocation
    // happens once, to gain a much faster impl for the middleware.

    // Key is literal URI path, value is the sanitized metrics name.
    // If no value, then it's not a path we want to instrument.
    // Map of known-good gRPC method routes.
    // metrics_keys: std::sync::Arc<BTreeMap<&'static str, &'static str>>,
    /// How many times any service has been called.
    // TODO: make this unique per-service; for now, just incrementing
    // a value to confirm the middleware hookup works.
    count: u64,
    // duration: std::time::Duration,
}

#[derive(Default, Debug, Clone)]
pub struct TonicMetricsLayer {
    count: u64,
    // metrics: TonicMetrics,
}

impl TonicMetricsLayer {
    #[instrument()]
    pub fn new() -> Self {
        Self::default()
    }
}
impl<S> tower::Layer<S> for TonicMetricsLayer
where
    S: std::fmt::Debug,
{
    type Service = TonicMetricsService<S>;

    #[instrument]
    fn layer(&self, service: S) -> Self::Service {
        Self::Service { count: 0, service }
    }
}

#[pin_project]
struct TonicMetricsFut<F> {
    /// The wrapped Future
    #[pin]
    inner: F,
    // for registering metrics:
    // start_time:
    // end_time:
}

impl<F> TonicMetricsFut<F>
where
    F: std::future::Future,
{
    // Not able to set `type Output` due to inherent associated types being unstable;
    // see for reference:
    //
    //   * E0658
    //   * https://github.com/rust-lang/rust/issues/8995
    //
    // type Output = std::futures::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        let this = self.project();
        // TODO: if we match on the result of polling the inner future,
        // we can hook extra code to run when the inner Future resolves.
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(rsp) => {
                // TODO: code that executes here will execute after the inner future resolves
                Poll::Ready(rsp)
            }
        }
    }
}

// We need to satisfy trait bounds for tracing-subscriber, so that we can use
// our custom Layer in pd, which uses tracing-subscriber layers.
// However, I'm not certain this impl actually works.
// impl<S> tracing_subscriber::layer::Layer<S> for TonicMetricsLayer where S: tracing::Subscriber {}

impl<S, Request> Service<Request> for TonicMetricsService<S>
where
    S: Service<Request> + std::fmt::Debug,
    Request: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[instrument]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::warn!("polling ready");
        self.service.poll_ready(cx)
    }

    #[instrument]
    fn call(&mut self, request: Request) -> Self::Future {
        // TODO: validate this is path we want to emit metrics for.
        // Logging a WARN to stand out in pd logs during dev spike.
        self.count = self.count + 1;
        tracing::warn!("JAWN request = {:?}, count = {:?}", request, self.count);
        self.service.call(request)
    }
}

/// Check whether a given URL path belongs to a known gRPC method.
/// If so, return the name of the corresponding Prometheus metric.
/// Otherwise, return None.
///
/// Since the requests made to the public pd endpoint constitute
/// attack-controlled data, we must vet those paths, lest we
/// pollute the metrics with arbitrary data.
fn convert_path_to_metrics_key(p: &str) -> anyhow::Result<Option<String>> {
    if !get_all_grpc_services()?.contains(&p.to_owned()) {
        return Ok(None);
    }
    let path = p.replace('.', "_").replace('/', "_");
    Ok(Some(format!("penumbra_grpc_method_{}", path)))
}

use penumbra_proto::FILE_DESCRIPTOR_SET;
use prost_reflect::{DescriptorPool, ServiceDescriptor};
/// Returns a Vec<String> where each String is a fully qualified gRPC query service name,
/// such as:
///
///   - penumbra.core.component.community_pool.v1.QueryService
///   - penumbra.view.v1.ViewService
///   - penumbra.core.component.dex.v1.SimulationService
///
/// The gRPC service names are read from the [penumbra_proto] crate's [FILE_DESCRIPTOR_SET],
/// which is exported at build time.
fn get_all_grpc_services() -> anyhow::Result<Vec<String>> {
    // Intentionally verbose to be explicit.
    let services: Vec<ServiceDescriptor> = DescriptorPool::decode(FILE_DESCRIPTOR_SET)?
        .services()
        .into_iter()
        .collect();
    let service_names: Vec<String> = services.iter().map(|x| x.full_name().to_owned()).collect();
    Ok(service_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Confirm that the URL path -> metrics key name transformation works,
    /// and also fails to work for an arbitrary path.
    fn path_sanitization() {
        // Happy path
        let method_name = "penumbra.core.component.community_pool.v1.QueryService".to_owned();
        let metrics_name = convert_path_to_metrics_key(&method_name).unwrap().unwrap();
        let expected =
            "penumbra_grpc_method_penumbra_core_component_community_pool_v1_QueryService"
                .to_owned();
        assert_eq!(metrics_name, expected);

        // Unhappy path
        let method_name = "path/to/service".to_owned();
        let metrics_name = convert_path_to_metrics_key(&method_name).unwrap();
        assert_eq!(metrics_name, None);
    }

    #[test]
    /// Check that the protobuf file descriptors can be converted to method names.
    /// This info is provided by the `penumbra-proto` crate at build time, and we want
    /// to validate its contents.
    fn file_descriptor_lookup() {
        let service_names = get_all_grpc_services().unwrap();
        // Spot-check a few hardcoded names
        assert!(service_names.contains(&"penumbra.view.v1.ViewService".to_string()));
        assert!(
            service_names.contains(&"penumbra.core.component.dex.v1.SimulationService".to_string())
        );
    }
}
