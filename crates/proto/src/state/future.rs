use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Context as _, Result};
use pin_project::pin_project;
use prost::Message;

use crate::DomainType;

/// A future that resolves to a protobuf message.
#[pin_project]
pub struct ProtoFuture<P, F> {
    #[pin]
    pub(super) inner: F,
    pub(super) _marker: std::marker::PhantomData<P>,
}

/// A future that resolves to a domain type.
#[pin_project]
pub struct DomainFuture<D, F> {
    #[pin]
    pub(super) inner: F,
    pub(super) _marker: std::marker::PhantomData<D>,
}

impl<F, P> Future for ProtoFuture<P, F>
where
    F: Future<Output = Result<Option<Vec<u8>>>>,
    P: Message + Default,
{
    type Output = Result<Option<P>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(Some(bytes))) => {
                let v = P::decode(&*bytes).context("could not decode proto from bytes")?;
                Poll::Ready(Ok(Some(v)))
            }
            Poll::Ready(Ok(None)) => Poll::Ready(Ok(None)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<D, F> Future for DomainFuture<D, F>
where
    F: Future<Output = Result<Option<Vec<u8>>>>,
    D: DomainType,
    anyhow::Error: From<<D as TryFrom<D::Proto>>::Error>,
{
    type Output = Result<Option<D>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(Some(bytes))) => {
                let v = D::Proto::decode(&*bytes).context("could not decode proto from bytes")?;
                let v = D::try_from(v)
                    .map_err(anyhow::Error::from)
                    .context("could not parse domain type from proto")?;
                Poll::Ready(Ok(Some(v)))
            }
            Poll::Ready(Ok(None)) => Poll::Ready(Ok(None)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
