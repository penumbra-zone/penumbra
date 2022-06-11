use bytes::Bytes;
use http_body::{combinators::UnsyncBoxBody, Body};
use tonic::{
    body::BoxBody as ReqBody,
    codegen::http as grpc,
    transport::{self, Endpoint},
};
use tower::{util::BoxCloneService, Service, ServiceBuilder};

/// A type-erased gRPC service.
pub(crate) type BoxGrpcService =
    BoxCloneService<grpc::Request<ReqBody>, grpc::Response<RspBody>, BoxError>;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A type-erased gRPC response [`Body`].
pub(crate) type RspBody = UnsyncBoxBody<Bytes, BoxError>;

/// Connects to the provided tonic [`Endpoint`], returning a [`BoxGrpcService`].
pub(crate) async fn connect(ep: Endpoint) -> anyhow::Result<BoxGrpcService> {
    let conn = ep.connect().await?;
    let svc = ServiceBuilder::new()
        .map_response(|rsp: grpc::Response<transport::Body>| rsp.map(box_rsp_body))
        .map_err(BoxError::from)
        .service(conn);
    Ok(BoxCloneService::new(svc))
}

/// Constructs a [`BoxGrpcService`] by erasing the type of an `S`-typed local
/// (in-process) service instance.
pub(crate) fn local<S, B>(svc: S) -> BoxGrpcService
where
    S: Service<grpc::Request<ReqBody>, Response = grpc::Response<B>>,
    S: Clone + Send + Sync + 'static,
    S::Error: 'static,
    S::Future: Send,
    BoxError: From<S::Error> + From<B::Error>,
    B: Body<Data = Bytes> + Send + 'static,
{
    let svc = ServiceBuilder::new()
        .map_response(|rsp: grpc::Response<B>| rsp.map(box_rsp_body))
        .map_err(BoxError::from)
        .service(svc);
    BoxCloneService::new(svc)
}

/// Erases a response body's `Error` type, returning a `RspBody`.
fn box_rsp_body<B>(body: B) -> RspBody
where
    B: Body<Data = Bytes> + Send + 'static,
    BoxError: From<B::Error>,
    B::Error: 'static,
{
    body.map_err(BoxError::from).boxed_unsync()
}
