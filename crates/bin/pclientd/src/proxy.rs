use futures::FutureExt;
use std::convert::Infallible;
use std::pin::Pin;
use std::{
    future::Future,
    task::{Context, Poll},
};
use tonic::server::NamedService;
use tonic::{body::BoxBody, transport::Channel};
use tower::ServiceExt;

fn proxy(
    channel: Channel,
    req: http::Request<BoxBody>,
) -> Pin<Box<dyn Future<Output = Result<http::Response<BoxBody>, Infallible>> + Send + 'static>> {
    tracing::debug!(headers = ?req.headers(), "proxying request");

    let rsp = channel.oneshot(req);

    async move {
        // Once we get the response, we need to convert any transport errors into
        // an Ok(HTTP response reporting an internal error), so we can have Error = Infallible
        let rsp = match rsp.await {
            Ok(rsp) => rsp,
            Err(e) => tonic::Status::internal(format!("grpc proxy error: {e}")).into_http(),
        };
        Ok::<_, Infallible>(rsp)
    }
    .boxed()
}

#[derive(Clone)]
pub struct AppQueryProxy(pub Channel);

impl NamedService for AppQueryProxy {
    const NAME: &'static str = "penumbra.core.app.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for AppQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct GovernanceQueryProxy(pub Channel);

impl NamedService for GovernanceQueryProxy {
    const NAME: &'static str = "penumbra.core.component.governance.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for GovernanceQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct DexQueryProxy(pub Channel);

impl NamedService for DexQueryProxy {
    const NAME: &'static str = "penumbra.core.component.dex.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for DexQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct DexSimulationProxy(pub Channel);

impl NamedService for DexSimulationProxy {
    const NAME: &'static str = "penumbra.core.component.dex.v1.SimulationService";
}

impl tower::Service<http::Request<BoxBody>> for DexSimulationProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct FeeQueryProxy(pub Channel);

impl NamedService for FeeQueryProxy {
    const NAME: &'static str = "penumbra.core.component.fee.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for FeeQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct SctQueryProxy(pub Channel);

impl NamedService for SctQueryProxy {
    const NAME: &'static str = "penumbra.core.component.sct.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for SctQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct ShieldedPoolQueryProxy(pub Channel);

impl NamedService for ShieldedPoolQueryProxy {
    const NAME: &'static str = "penumbra.core.component.shielded_pool.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for ShieldedPoolQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct ChainQueryProxy(pub Channel);

impl NamedService for ChainQueryProxy {
    const NAME: &'static str = "penumbra.core.component.chain.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for ChainQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct StakeQueryProxy(pub Channel);

impl NamedService for StakeQueryProxy {
    const NAME: &'static str = "penumbra.core.component.stake.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for StakeQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct CompactBlockQueryProxy(pub Channel);

impl NamedService for CompactBlockQueryProxy {
    const NAME: &'static str = "penumbra.core.component.compact_block.v1.QueryService";
}

impl tower::Service<http::Request<BoxBody>> for CompactBlockQueryProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}

#[derive(Clone)]
pub struct TendermintProxyProxy(pub Channel);

impl NamedService for TendermintProxyProxy {
    const NAME: &'static str = "penumbra.util.tendermint_proxy.v1.TendermintProxyService";
}

impl tower::Service<http::Request<BoxBody>> for TendermintProxyProxy {
    type Response = http::Response<BoxBody>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        proxy(self.0.clone(), req)
    }
}
