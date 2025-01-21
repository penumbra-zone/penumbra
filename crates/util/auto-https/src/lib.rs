//! Automatic HTTPS certificate management facilities.
//!
//! See [`axum_acceptor`] for more information.

use {
    anyhow::Error,
    futures::Future,
    rustls::ServerConfig,
    rustls_acme::{axum::AxumAcceptor, caches::DirCache, AcmeConfig, AcmeState},
    std::{fmt::Debug, path::PathBuf, sync::Arc},
};

/// Protocols supported by this server, in order of preference.
///
/// See [rfc7301] for more info on ALPN.
///
/// [rfc7301]: https://datatracker.ietf.org/doc/html/rfc7301
//
//  We also permit HTTP1.1 for backwards-compatibility, specifically for grpc-web.
const ALPN_PROTOCOLS: [&[u8]; 2] = [b"h2", b"http/1.1"];

/// The location of the file-based certificate cache.
//  NB: this must not be an absolute path see [Path::join].
const CACHE_DIR: &str = "tokio_rustls_acme_cache";

/// Use ACME to resolve certificates and handle new connections.
///
/// This returns a tuple containing an [`AxumAcceptor`] that may be used with [`axum_server`], and
/// a [`Future`] that represents the background task to poll and log for changes in the
/// certificate environment.
pub fn axum_acceptor(
    home: PathBuf,
    domain: String,
    production_api: bool,
) -> (AxumAcceptor, impl Future<Output = Result<(), Error>>) {
    // Use a file-based cache located within the home directory.
    let cache = home.join(CACHE_DIR);
    let cache = DirCache::new(cache);

    // Create an ACME client, which we will use to resolve certificates.
    let state = AcmeConfig::new(vec![domain])
        .cache(cache)
        .directory_lets_encrypt(production_api)
        .state();

    // Define our server configuration, using the ACME certificate resolver.
    let mut rustls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(state.resolver());
    rustls_config.alpn_protocols = self::alpn_protocols();
    let rustls_config = Arc::new(rustls_config);

    // Return our connection acceptor and our background worker task.
    let acceptor = state.axum_acceptor(rustls_config.clone());
    let worker = self::acme_worker(state);
    (acceptor, worker)
}

/// This function defines the task responsible for handling ACME events.
///
/// This function will never return, unless an error is encountered.
#[tracing::instrument(level = "error", skip_all)]
async fn acme_worker<EC, EA>(mut state: AcmeState<EC, EA>) -> Result<(), anyhow::Error>
where
    EC: Debug + 'static,
    EA: Debug + 'static,
{
    use futures::StreamExt;
    loop {
        match state.next().await {
            Some(Ok(ok)) => tracing::debug!("received acme event: {:?}", ok),
            Some(Err(err)) => {
                tracing::error!("acme error: {:?}", err);
                anyhow::bail!("exiting due to acme error");
            }
            None => {
                debug_assert!(false, "acme worker unexpectedly reached end-of-stream");
                tracing::error!("acme worker unexpectedly reached end-of-stream");
                anyhow::bail!("unexpected end-of-stream");
            }
        }
    }
}

/// Returns a vector of the protocols supported by this server.
///
/// This is a convenience method to retrieve an owned copy of [`ALPN_PROTOCOLS`].
fn alpn_protocols() -> Vec<Vec<u8>> {
    ALPN_PROTOCOLS.into_iter().map(<[u8]>::to_vec).collect()
}
