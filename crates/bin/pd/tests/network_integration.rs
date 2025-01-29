//! Basic integration testing for pd.
//!
//! Validates behavior of the pd binary itself, such as serving specific HTTP
//! headers in all contexts. Does NOT evaluate application logic; see the
//! integration tests for pcli/pclientd for that.

use assert_cmd::Command;
use http::StatusCode;
use penumbra_sdk_proto::FILE_DESCRIPTOR_SET;
use predicates::prelude::*;
use prost_reflect::{DescriptorPool, ServiceDescriptor};
use regex::Regex;
use rstest::rstest;
use url::Url;

#[rstest]
/// Specific patterns for spot-checking the metrics emitted by pd.
/// It's a smattering of metrics from the various components, including
/// some from outside the workspace, e.g. `cnidarium`.
#[case(r"^cnidarium_get_raw_duration_seconds_count_seconds \d+")]
#[case(r"^cnidarium_nonverifiable_get_raw_duration_seconds_count_seconds \d+")]
#[case(r"^pd_async_sleep_drift_microseconds \d+")]
#[case(r"^pd_process_cpu_seconds_total \d+")]
#[case(r"^pd_process_open_fds \d+")]
#[case(r#"^penumbra_stake_missed_blocks\{identity_key=".*"\} \d+"#)]
#[case(r"^penumbra_funding_streams_total_processing_time_milliseconds_count_milliseconds \d+")]
#[case(r"^penumbra_dex_path_search_duration_seconds_count_seconds \d+")]
#[case(r"^penumbra_dex_path_search_relax_path_duration_seconds_count_seconds \d+")]
#[tokio::test]
#[ignore]
/// Confirm that prometheus metrics are being exported for scraping.
/// Several times while bumping related crates we've missed a breakage
/// to metrics, and only noticed when we checked the grafana boards
/// for the preview environment post-deploy.
async fn confirm_metrics_emission(#[case] pattern: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let metrics_url = std::env::var("PENUMBRA_NODE_PD_METRICS_URL")
        .unwrap_or("http://localhost:9000/metrics".to_string());
    let r = client.get(metrics_url).send().await?;
    let status = r.status();
    let body = r.text().await?;
    // Assert 200
    assert_eq!(status, StatusCode::OK);

    // Enable multi-line support in the regex matching.
    let re = Regex::new(&format!(r"(?m){}", pattern))?;
    assert!(re.is_match(&body), "pd metric missing: {}", pattern);

    Ok(())
}

#[ignore]
#[tokio::test]
/// Confirm that permissive CORS headers are returned in HTTP responses
/// by pd. We want these headers to be served directly by pd, so that
/// an intermediate reverse-proxy is not required.
async fn check_cors_headers() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let pd_url =
        std::env::var("PENUMBRA_NODE_PD_URL").unwrap_or("http://localhost:8080".to_string());
    let r = client.get(pd_url).send().await?;
    assert_eq!(r.headers().get("access-control-allow-origin").unwrap(), "*");
    assert_eq!(
        r.headers().get("access-control-expose-headers").unwrap(),
        "*"
    );
    Ok(())
}

#[ignore]
#[tokio::test]
/// Confirm that the naive GET on the gRPC route returns a 200,
/// as a sanity check that we haven't badly broken the minifront static asset bundling.
/// This check does *not* confirm that page works correctly, but it does confirm
/// it's at least loading, which guards against path regressions in the asset building.
/// See GH4139 for context.
async fn check_minifront_http_ok() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let pd_url =
        std::env::var("PENUMBRA_NODE_PD_URL").unwrap_or("http://localhost:8080".to_string());
    let r = client.get(pd_url).send().await?;
    assert_eq!(r.status(), StatusCode::OK);
    Ok(())
}

#[ignore]
#[tokio::test]
/// Validate that gRPC server reflection is enabled and working, by calling out
/// to `grpcurl` and verifying that it can view methods. See GH4392 for context.
async fn check_grpc_server_reflection() -> anyhow::Result<()> {
    let pd_url: Url = std::env::var("PENUMBRA_NODE_PD_URL")
        .unwrap_or("http://localhost:8080".to_string())
        .parse()
        .unwrap();
    let pd_hostname = format!("{}:{}", pd_url.host().unwrap(), pd_url.port().unwrap());
    let mut args = Vec::<String>::new();
    if pd_url.scheme() == "http" {
        args.push("-plaintext".to_owned());
    }
    args.push(pd_hostname);
    // grpcurl takes `list` as a command, to inspect the server reflection API.
    args.push("list".to_owned());

    // Permit override of the fullpath to the `grpcurl` binary, in case we want
    // to test multiple versions in CI.
    let grpcurl_path = std::env::var("GRPCURL_PATH").unwrap_or("grpcurl".to_string());
    let std_cmd = std::process::Command::new(grpcurl_path);
    let mut cmd = Command::from_std(std_cmd);
    cmd.args(args);

    // Here we hardcode a few specific checks, to verify they're present.
    // This ensures reflection is ostensibly working, and doesn't assume
    // that the FILE_DESCRIPTOR tonic-build logic is wired up.
    let methods = vec![
        "penumbra.core.app.v1.QueryService",
        // "grpc.reflection.v1alpha.ServerReflection",
        "grpc.reflection.v1.ServerReflection",
        "ibc.core.channel.v1.Query",
    ];
    for m in methods {
        cmd.assert().stdout(predicate::str::contains(m));
    }

    // Here we look up the gRPC services exported from the proto crate,
    // as FILE_DESCRIPTOR_SET. All of these methods should be visible
    // to the `grpcurl` list command, if reflection is working.
    let grpc_service_names = get_all_grpc_services()?;
    // Sanity-check that we actually got results.
    assert!(grpc_service_names.len() > 5);
    for m in grpc_service_names {
        cmd.assert().stdout(predicate::str::contains(m));
    }
    Ok(())
}

/// Returns a Vec<String> where each String is a fully qualified gRPC query service name,
/// such as:
///
///   - penumbra.core.component.community_pool.v1.QueryService
///   - penumbra.view.v1.ViewService
///   - penumbra.core.component.dex.v1.SimulationService
///
/// The gRPC service names are read from the [penumbra_sdk_proto] crate's [FILE_DESCRIPTOR_SET],
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
