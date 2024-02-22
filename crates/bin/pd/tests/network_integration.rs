//! Basic integration testing for pd.
//!
//! Validates behavior of the pd binary itself, such as serving specific HTTP
//! headers in all contexts. Does NOT evaluate application logic; see the
//! integration tests for pcli/pclientd for that.

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
