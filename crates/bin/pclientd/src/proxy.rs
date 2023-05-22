use futures::FutureExt;
use http_body::Body as _;
use std::convert::Infallible;
use std::pin::Pin;
use std::{
    future::Future,
    task::{Context, Poll},
};
use tonic::transport::NamedService;
use tonic::{
    body::BoxBody,
    transport::{Body, Channel},
};
use tower::ServiceExt;

#[derive(Clone)]
pub struct ObliviousQueryProxy(pub Channel);

impl NamedService for ObliviousQueryProxy {
    const NAME: &'static str = "penumbra.client.v1alpha1.ObliviousQueryService";
}

fn proxy(
    channel: Channel,
    req: http::Request<Body>,
) -> Pin<Box<dyn Future<Output = Result<http::Response<BoxBody>, Infallible>> + Send + 'static>> {
    tracing::debug!(headers = ?req.headers(), "proxying request");
    // Convert request types
    let req = req.map(|b| {
        b.map_err(|e| tonic::Status::from_error(Box::new(e)))
            .boxed_unsync()
    });

    let rsp = channel.oneshot(req);

    async move {
        // Once we get the response, we need to convert any transport errors into
        // an Ok(HTTP response reporting an internal error), so we can have Error = Infallible
        let rsp = match rsp.await {
            Ok(rsp) => rsp.map(|b| {
                b.map_err(|e| tonic::Status::from_error(Box::new(e)))
                    .boxed_unsync()
            }),
            Err(e) => tonic::Status::internal(format!("grpc proxy error: {e}")).to_http(),
        };
        Ok::<_, Infallible>(rsp)
    }
    .boxed()
}

impl tower::Service<http::Request<Body>> for ObliviousQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct SpecificQueryProxy(pub Channel);

impl NamedService for SpecificQueryProxy {
    const NAME: &'static str = "penumbra.client.v1alpha1.SpecificQueryService";
}

impl tower::Service<http::Request<Body>> for SpecificQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct TendermintProxyProxy(pub Channel);

impl NamedService for TendermintProxyProxy {
    const NAME: &'static str = "penumbra.client.v1alpha1.TendermintProxyService";
}

impl tower::Service<http::Request<Body>> for TendermintProxyProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}
