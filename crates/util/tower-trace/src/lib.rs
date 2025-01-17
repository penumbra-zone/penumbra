#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// A vendored copy of the unpublished `tracing-tower` crate.
pub mod trace;

use std::net::SocketAddr;

// Extracted from tonic's remote_addr implementation; we'd like to instrument
// spans with the remote addr at the server level rather than at the individual
// request level, but the hook available to do that gives us an http::Request
// rather than a tonic::Request, so the tonic::Request::remote_addr method isn't
// available.
pub fn remote_addr<B>(req: &http::Request<B>) -> Option<SocketAddr> {
    use tonic::transport::server::TcpConnectInfo;
    // NOTE: needs to also check TlsConnectInfo if we use TLS
    req.extensions()
        .get::<TcpConnectInfo>()
        .and_then(|i| i.remote_addr())
}

pub mod v034 {
    mod request_ext;
    pub use request_ext::RequestExt;
}

pub mod v037 {
    mod request_ext;
    pub use request_ext::RequestExt;
}

pub mod v038 {
    mod request_ext;
    pub use request_ext::RequestExt;
}
