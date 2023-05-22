use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tendermint::abci::{SnapshotRequest, SnapshotResponse};
use tower_abci::BoxError;

#[derive(Clone, Debug)]
pub struct Snapshot {}

impl tower_service::Service<SnapshotRequest> for Snapshot {
    type Response = SnapshotResponse;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<SnapshotResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SnapshotRequest) -> Self::Future {
        // No-op, we don't implement snapshot support
        use SnapshotRequest as Request;
        use SnapshotResponse as Response;
        async move {
            Ok(match req {
                Request::ListSnapshots => Response::ListSnapshots(Default::default()),
                Request::OfferSnapshot(_) => Response::OfferSnapshot(Default::default()),
                Request::LoadSnapshotChunk(_) => Response::LoadSnapshotChunk(Default::default()),
                Request::ApplySnapshotChunk(_) => Response::ApplySnapshotChunk(Default::default()),
            })
        }
        .boxed()
    }
}
