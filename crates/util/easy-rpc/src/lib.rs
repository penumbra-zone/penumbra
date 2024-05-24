use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use tokio::task::JoinHandle;

/// A lightweight wrapper around a `JoinHandle` that will cancel the task when dropped.
pub struct CancelOnDrop<T>(JoinHandle<T>);

impl<T> Drop for CancelOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// A stream with an associated worker task that will be
/// cancelled when the stream is dropped.
pub struct StreamWithWorker<S, T> {
    stream: S,
    _cancel: CancelOnDrop<T>,
}

impl<S, T> StreamWithWorker<S, T> {
    pub fn new(stream: S, cancel_on_drop: JoinHandle<T>) -> Self {
        Self {
            stream,
            _cancel: CancelOnDrop(cancel_on_drop),
        }
    }
}

impl<S, T> Stream for StreamWithWorker<S, T>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.stream).poll_next(cx)
    }
}
