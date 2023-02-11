//! Concrete futures types used by the storage crate.

use anyhow::Result;
use futures::future::{Either, Ready};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Future representing a read from a state snapshot.
#[pin_project]
pub struct SnapshotFuture(#[pin] pub(crate) tokio::task::JoinHandle<Result<Option<Vec<u8>>>>);

impl Future for SnapshotFuture {
    type Output = Result<Option<Vec<u8>>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            Poll::Ready(result) => Poll::Ready(result.unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Future representing a read from an in-memory cache over an underlying state.
#[pin_project]
pub struct CacheFuture<F> {
    #[pin]
    inner: Either<Ready<Result<Option<Vec<u8>>>>, F>,
}

impl<F> CacheFuture<F> {
    pub(crate) fn hit(value: Option<Vec<u8>>) -> Self {
        Self {
            inner: Either::Left(futures::future::ready(Ok(value))),
        }
    }

    pub(crate) fn miss(underlying: F) -> Self {
        Self {
            inner: Either::Right(underlying),
        }
    }
}

impl<F> Future for CacheFuture<F>
where
    F: Future<Output = Result<Option<Vec<u8>>>>,
{
    type Output = Result<Option<Vec<u8>>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
