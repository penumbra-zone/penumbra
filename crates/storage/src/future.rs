//! Concrete futures types used by the storage crate.

use anyhow::Result;
use futures::{
    future::{Either, Ready},
    stream::Peekable,
    Stream,
};
use parking_lot::RwLock;
use pin_project::pin_project;
use smallvec::SmallVec;
use std::{
    future::Future,
    ops::Bound,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

use crate::Cache;

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

#[pin_project]
pub struct StateDeltaNonconsensusPrefixRawStream<St>
where
    St: Stream<Item = Result<(Vec<u8>, Vec<u8>)>>,
{
    #[pin]
    pub(crate) underlying: Peekable<St>,
    pub(crate) layers: Vec<Arc<RwLock<Option<Cache>>>>,
    pub(crate) leaf_cache: Arc<RwLock<Option<Cache>>>,
    pub(crate) last_key: Option<Vec<u8>>,
    pub(crate) prefix: Vec<u8>,
}

impl<St> Stream for StateDeltaNonconsensusPrefixRawStream<St>
where
    St: Stream<Item = Result<(Vec<u8>, Vec<u8>)>>,
{
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // This implementation interleaves items from the underlying stream with
        // items in cache layers.  To do this, it tracks the last key it
        // returned, then, for each item in the underlying stream, searches for
        // cached keys that lie between the last-returned key and the item's key,
        // checking whether the cached key represents a deletion requiring further
        // scanning.  This process is illustrated as follows:
        //
        //         ◇ skip                 ◇ skip           ▲ yield          ▲ yield           ▲ yield
        //         │                      │                │                │                 │
        //         ░ pick ──────────────▶ ░ pick ────────▶ █ pick ────────▶ █ pick ─────────▶ █ pick
        //         ▲                      ▲                ▲                ▲                 ▲
        //      ▲  │                 ▲    │          ▲     │         ▲      │        ▲        │
        // write│  │                 │    │          │     │         │      │        │        │
        // layer│  │   █             │    │ █        │     │█        │      █        │      █ │
        //      │  │ ░               │    ░          │    ░│         │    ░          │    ░   │
        //      │  ░                 │  ░            │  ░  │         │  ░            │  ░     │
        //      │    █               │    █          │    █│         │    █          │    █   │
        //      │  █     █           │  █     █      │  █  │  █      │  █     █      │  █     █
        //      │     █              │     █         │     █         │     █         │     █
        //     ─┼(─────]────keys─▶  ─┼──(───]────▶  ─┼────(─]────▶  ─┼─────(]────▶  ─┼──────(──]─▶
        //      │   ▲  █  █          │      █  █     │      █  █     │      █  █     │      █  █
        //          │
        //          │search range of key-value pairs in cache layers that could
        //          │affect whether to yield the next item in the underlying stream

        // Optimization: ensure we have a peekable item in the underlying stream before continuing.
        let mut this = self.project();
        ready!(this.underlying.as_mut().poll_peek(cx));

        // Now that we're ready to interleave the next underlying item with any
        // cache layers, lock them all for the duration of the method, using a
        // SmallVec to (hopefully) store all the guards on the stack.
        let mut layer_guards = SmallVec::<[_; 8]>::new();
        for layer in this.layers.iter() {
            layer_guards.push(layer.read());
        }
        // Tacking the leaf cache onto the list is important to not miss any values.
        // It's stored separately so that the contents of the
        layer_guards.push(this.leaf_cache.read());

        loop {
            // Obtain a reference to the next key-value pair from the underlying stream.
            let peeked = match ready!(this.underlying.as_mut().poll_peek(cx)) {
                // If we get an underlying error, bubble it up immediately.
                Some(Err(_e)) => return this.underlying.poll_next(cx),
                // Otherwise, pass through the peeked value.
                Some(Ok(pair)) => Some(pair),
                None => None,
            };

            // To determine whether or not we should return the peeked value, we
            // need to search the cache layers for keys that are between the last
            // key we returned (exclusive, so we make forward progress on the
            // stream) and the peeked key (inclusive, because we need to find out
            // whether or not there was a covering deletion).
            let search_range = (
                this.last_key
                    .as_ref()
                    .map(Bound::Excluded)
                    .unwrap_or(Bound::Included(this.prefix)),
                peeked
                    .map(|(k, _)| Bound::Included(k))
                    .unwrap_or(Bound::Unbounded),
            );

            // It'd be slightly cleaner to initialize `leftmost_pair` with the
            // peeked contents, but that would taint `leftmost_pair` with a
            // `peeked` borrow, and we may need to mutate the underlying stream
            // later.  Instead, initialize it with `None` to only search the
            // cache layers, and compare at the end.
            let mut leftmost_pair = None;
            for layer in layer_guards.iter() {
                // Find this layer's leftmost key-value pair in the search range.
                let found_pair = layer
                    .as_ref()
                    .unwrap()
                    .nonconsensus_changes
                    .range::<Vec<u8>, _>(search_range)
                    .take_while(|(k, _v)| k.starts_with(this.prefix))
                    .next();

                // Check whether the new pair, if any, is the new leftmost pair.
                match (leftmost_pair, found_pair) {
                    // We want to replace the pair even when the key is equal,
                    // so that we always prefer a newer value over an older value.
                    (Some((leftmost_k, _)), Some((k, v))) if k <= leftmost_k => {
                        leftmost_pair = Some((k, v));
                    }
                    (None, Some((k, v))) => {
                        leftmost_pair = Some((k, v));
                    }
                    _ => {}
                }
            }

            // Overwrite a Vec, attempting to reuse its existing allocation.
            let overwrite_in_place = |dst: &mut Option<Vec<u8>>, src: &[u8]| {
                if let Some(ref mut dst) = dst {
                    dst.clear();
                    dst.extend_from_slice(src);
                } else {
                    *dst = Some(src.to_vec());
                }
            };

            match (leftmost_pair, peeked) {
                (Some((k, v)), peeked) => {
                    // Since we searched for cached keys less than or equal to
                    // the peeked key, we know that the cached pair takes
                    // priority over the peeked pair.
                    //
                    // If the keys are exactly equal, we advance the underlying stream.
                    if peeked.map(|(kp, _)| kp) == Some(k) {
                        let _ = this.underlying.as_mut().poll_next(cx);
                    }
                    overwrite_in_place(this.last_key, k);
                    if let Some(v) = v {
                        // If the value is Some, we have a key-value pair to yield.
                        return Poll::Ready(Some(Ok((k.clone(), v.clone()))));
                    } else {
                        // If the value is None, this pair represents a deletion,
                        // so continue looping until we find a non-deleted pair.
                        continue;
                    }
                }
                (None, Some(_)) => {
                    // There's no cache hit before the peeked pair, so we want
                    // to extract and return it from the underlying stream.
                    let Poll::Ready(Some(Ok((k, v)))) = this.underlying.as_mut().poll_next(cx) else {
                        unreachable!("peeked stream must yield peeked item");
                    };
                    overwrite_in_place(this.last_key, &k);
                    return Poll::Ready(Some(Ok((k, v))));
                }
                (None, None) => {
                    // Terminate the stream, no more items are available.
                    return Poll::Ready(None);
                }
            }
        }
    }
}

// This implementation is almost exactly the same as the one above, but with
// minor tweaks to work with string keys and to read different fields from the cache.
// Update them together.

#[pin_project]
pub struct StateDeltaPrefixRawStream<St>
where
    St: Stream<Item = Result<(String, Vec<u8>)>>,
{
    #[pin]
    pub(crate) underlying: Peekable<St>,
    pub(crate) layers: Vec<Arc<RwLock<Option<Cache>>>>,
    pub(crate) leaf_cache: Arc<RwLock<Option<Cache>>>,
    pub(crate) last_key: Option<String>,
    pub(crate) prefix: String,
}

impl<St> Stream for StateDeltaPrefixRawStream<St>
where
    St: Stream<Item = Result<(String, Vec<u8>)>>,
{
    type Item = Result<(String, Vec<u8>)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // This implementation interleaves items from the underlying stream with
        // items in cache layers.  To do this, it tracks the last key it
        // returned, then, for each item in the underlying stream, searches for
        // cached keys that lie between the last-returned key and the item's key,
        // checking whether the cached key represents a deletion requiring further
        // scanning.  This process is illustrated as follows:
        //
        //         ◇ skip                 ◇ skip           ▲ yield          ▲ yield           ▲ yield
        //         │                      │                │                │                 │
        //         ░ pick ──────────────▶ ░ pick ────────▶ █ pick ────────▶ █ pick ─────────▶ █ pick
        //         ▲                      ▲                ▲                ▲                 ▲
        //      ▲  │                 ▲    │          ▲     │         ▲      │        ▲        │
        // write│  │                 │    │          │     │         │      │        │        │
        // layer│  │   █             │    │ █        │     │█        │      █        │      █ │
        //      │  │ ░               │    ░          │    ░│         │    ░          │    ░   │
        //      │  ░                 │  ░            │  ░  │         │  ░            │  ░     │
        //      │    █               │    █          │    █│         │    █          │    █   │
        //      │  █     █           │  █     █      │  █  │  █      │  █     █      │  █     █
        //      │     █              │     █         │     █         │     █         │     █
        //     ─┼(─────]────keys─▶  ─┼──(───]────▶  ─┼────(─]────▶  ─┼─────(]────▶  ─┼──────(──]─▶
        //      │   ▲  █  █          │      █  █     │      █  █     │      █  █     │      █  █
        //          │
        //          │search range of key-value pairs in cache layers that could
        //          │affect whether to yield the next item in the underlying stream

        // Optimization: ensure we have a peekable item in the underlying stream before continuing.
        let mut this = self.project();
        ready!(this.underlying.as_mut().poll_peek(cx));

        // Now that we're ready to interleave the next underlying item with any
        // cache layers, lock them all for the duration of the method, using a
        // SmallVec to (hopefully) store all the guards on the stack.
        let mut layer_guards = SmallVec::<[_; 8]>::new();
        for layer in this.layers.iter() {
            layer_guards.push(layer.read());
        }
        // Tacking the leaf cache onto the list is important to not miss any values.
        // It's stored separately so that the contents of the
        layer_guards.push(this.leaf_cache.read());

        loop {
            // Obtain a reference to the next key-value pair from the underlying stream.
            let peeked = match ready!(this.underlying.as_mut().poll_peek(cx)) {
                // If we get an underlying error, bubble it up immediately.
                Some(Err(_e)) => return this.underlying.poll_next(cx),
                // Otherwise, pass through the peeked value.
                Some(Ok(pair)) => Some(pair),
                None => None,
            };

            // To determine whether or not we should return the peeked value, we
            // need to search the cache layers for keys that are between the last
            // key we returned (exclusive, so we make forward progress on the
            // stream) and the peeked key (inclusive, because we need to find out
            // whether or not there was a covering deletion).
            let search_range = (
                this.last_key
                    .as_ref()
                    .map(Bound::Excluded)
                    .unwrap_or(Bound::Included(this.prefix)),
                peeked
                    .map(|(k, _)| Bound::Included(k))
                    .unwrap_or(Bound::Unbounded),
            );

            // It'd be slightly cleaner to initialize `leftmost_pair` with the
            // peeked contents, but that would taint `leftmost_pair` with a
            // `peeked` borrow, and we may need to mutate the underlying stream
            // later.  Instead, initialize it with `None` to only search the
            // cache layers, and compare at the end.
            let mut leftmost_pair = None;
            for layer in layer_guards.iter() {
                // Find this layer's leftmost key-value pair in the search range.
                let found_pair = layer
                    .as_ref()
                    .unwrap()
                    .unwritten_changes
                    .range::<String, _>(search_range)
                    .take_while(|(k, _v)| k.starts_with(this.prefix.as_str()))
                    .next();

                // Check whether the new pair, if any, is the new leftmost pair.
                match (leftmost_pair, found_pair) {
                    // We want to replace the pair even when the key is equal,
                    // so that we always prefer a newer value over an older value.
                    (Some((leftmost_k, _)), Some((k, v))) if k <= leftmost_k => {
                        leftmost_pair = Some((k, v));
                    }
                    (None, Some((k, v))) => {
                        leftmost_pair = Some((k, v));
                    }
                    _ => {}
                }
            }

            // Overwrite a Vec, attempting to reuse its existing allocation.
            let overwrite_in_place = |dst: &mut Option<String>, src: &str| {
                if let Some(ref mut dst) = dst {
                    dst.clear();
                    dst.push_str(src);
                } else {
                    *dst = Some(src.to_owned());
                }
            };

            match (leftmost_pair, peeked) {
                (Some((k, v)), peeked) => {
                    // Since we searched for cached keys less than or equal to
                    // the peeked key, we know that the cached pair takes
                    // priority over the peeked pair.
                    //
                    // If the keys are exactly equal, we advance the underlying stream.
                    if peeked.map(|(kp, _)| kp) == Some(k) {
                        let _ = this.underlying.as_mut().poll_next(cx);
                    }
                    overwrite_in_place(this.last_key, k);
                    if let Some(v) = v {
                        // If the value is Some, we have a key-value pair to yield.
                        return Poll::Ready(Some(Ok((k.clone(), v.clone()))));
                    } else {
                        // If the value is None, this pair represents a deletion,
                        // so continue looping until we find a non-deleted pair.
                        continue;
                    }
                }
                (None, Some(_)) => {
                    // There's no cache hit before the peeked pair, so we want
                    // to extract and return it from the underlying stream.
                    let Poll::Ready(Some(Ok((k, v)))) = this.underlying.as_mut().poll_next(cx) else {
                        unreachable!("peeked stream must yield peeked item");
                    };
                    overwrite_in_place(this.last_key, &k);
                    return Poll::Ready(Some(Ok((k, v))));
                }
                (None, None) => {
                    // Terminate the stream, no more items are available.
                    return Poll::Ready(None);
                }
            }
        }
    }
}

// This implementation is almost exactly the same as the one above, but with
// minor tweaks to work with string keys and to read different fields from the cache.
// Update them together.

#[pin_project]
pub struct StateDeltaPrefixKeysStream<St>
where
    St: Stream<Item = Result<String>>,
{
    #[pin]
    pub(crate) underlying: Peekable<St>,
    pub(crate) layers: Vec<Arc<RwLock<Option<Cache>>>>,
    pub(crate) leaf_cache: Arc<RwLock<Option<Cache>>>,
    pub(crate) last_key: Option<String>,
    pub(crate) prefix: String,
}

impl<St> Stream for StateDeltaPrefixKeysStream<St>
where
    St: Stream<Item = Result<String>>,
{
    type Item = Result<String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // This implementation interleaves items from the underlying stream with
        // items in cache layers.  To do this, it tracks the last key it
        // returned, then, for each item in the underlying stream, searches for
        // cached keys that lie between the last-returned key and the item's key,
        // checking whether the cached key represents a deletion requiring further
        // scanning.  This process is illustrated as follows:
        //
        //         ◇ skip                 ◇ skip           ▲ yield          ▲ yield           ▲ yield
        //         │                      │                │                │                 │
        //         ░ pick ──────────────▶ ░ pick ────────▶ █ pick ────────▶ █ pick ─────────▶ █ pick
        //         ▲                      ▲                ▲                ▲                 ▲
        //      ▲  │                 ▲    │          ▲     │         ▲      │        ▲        │
        // write│  │                 │    │          │     │         │      │        │        │
        // layer│  │   █             │    │ █        │     │█        │      █        │      █ │
        //      │  │ ░               │    ░          │    ░│         │    ░          │    ░   │
        //      │  ░                 │  ░            │  ░  │         │  ░            │  ░     │
        //      │    █               │    █          │    █│         │    █          │    █   │
        //      │  █     █           │  █     █      │  █  │  █      │  █     █      │  █     █
        //      │     █              │     █         │     █         │     █         │     █
        //     ─┼(─────]────keys─▶  ─┼──(───]────▶  ─┼────(─]────▶  ─┼─────(]────▶  ─┼──────(──]─▶
        //      │   ▲  █  █          │      █  █     │      █  █     │      █  █     │      █  █
        //          │
        //          │search range of key-value pairs in cache layers that could
        //          │affect whether to yield the next item in the underlying stream

        // Optimization: ensure we have a peekable item in the underlying stream before continuing.
        let mut this = self.project();
        ready!(this.underlying.as_mut().poll_peek(cx));

        // Now that we're ready to interleave the next underlying item with any
        // cache layers, lock them all for the duration of the method, using a
        // SmallVec to (hopefully) store all the guards on the stack.
        let mut layer_guards = SmallVec::<[_; 8]>::new();
        for layer in this.layers.iter() {
            layer_guards.push(layer.read());
        }
        // Tacking the leaf cache onto the list is important to not miss any values.
        // It's stored separately so that the contents of the
        layer_guards.push(this.leaf_cache.read());

        loop {
            // Obtain a reference to the next key-value pair from the underlying stream.
            let peeked = match ready!(this.underlying.as_mut().poll_peek(cx)) {
                // If we get an underlying error, bubble it up immediately.
                Some(Err(_e)) => return this.underlying.poll_next(cx),
                // Otherwise, pass through the peeked value.
                Some(Ok(pair)) => Some(pair),
                None => None,
            };

            // To determine whether or not we should return the peeked value, we
            // need to search the cache layers for keys that are between the last
            // key we returned (exclusive, so we make forward progress on the
            // stream) and the peeked key (inclusive, because we need to find out
            // whether or not there was a covering deletion).
            let search_range = (
                this.last_key
                    .as_ref()
                    .map(Bound::Excluded)
                    .unwrap_or(Bound::Included(this.prefix)),
                peeked.map(Bound::Included).unwrap_or(Bound::Unbounded),
            );

            // It'd be slightly cleaner to initialize `leftmost_pair` with the
            // peeked contents, but that would taint `leftmost_pair` with a
            // `peeked` borrow, and we may need to mutate the underlying stream
            // later.  Instead, initialize it with `None` to only search the
            // cache layers, and compare at the end.
            let mut leftmost_pair = None;
            for layer in layer_guards.iter() {
                // Find this layer's leftmost key-value pair in the search range.
                let found_pair = layer
                    .as_ref()
                    .unwrap()
                    .unwritten_changes
                    .range::<String, _>(search_range)
                    .take_while(|(k, _v)| k.starts_with(this.prefix.as_str()))
                    .next();

                // Check whether the new pair, if any, is the new leftmost pair.
                match (leftmost_pair, found_pair) {
                    // We want to replace the pair even when the key is equal,
                    // so that we always prefer a newer value over an older value.
                    (Some((leftmost_k, _)), Some((k, v))) if k <= leftmost_k => {
                        leftmost_pair = Some((k, v));
                    }
                    (None, Some((k, v))) => {
                        leftmost_pair = Some((k, v));
                    }
                    _ => {}
                }
            }

            // Overwrite a Vec, attempting to reuse its existing allocation.
            let overwrite_in_place = |dst: &mut Option<String>, src: &str| {
                if let Some(ref mut dst) = dst {
                    dst.clear();
                    dst.push_str(src);
                } else {
                    *dst = Some(src.to_owned());
                }
            };

            match (leftmost_pair, peeked) {
                (Some((k, v)), peeked) => {
                    // Since we searched for cached keys less than or equal to
                    // the peeked key, we know that the cached pair takes
                    // priority over the peeked pair.
                    //
                    // If the keys are exactly equal, we advance the underlying stream.
                    if peeked == Some(k) {
                        let _ = this.underlying.as_mut().poll_next(cx);
                    }
                    overwrite_in_place(this.last_key, k);
                    if v.is_some() {
                        // If the value is Some, we have a key-value pair to yield.
                        return Poll::Ready(Some(Ok(k.clone())));
                    } else {
                        // If the value is None, this pair represents a deletion,
                        // so continue looping until we find a non-deleted pair.
                        continue;
                    }
                }
                (None, Some(_)) => {
                    // There's no cache hit before the peeked pair, so we want
                    // to extract and return it from the underlying stream.
                    let Poll::Ready(Some(Ok(k))) = this.underlying.as_mut().poll_next(cx) else {
                        unreachable!("peeked stream must yield peeked item");
                    };
                    overwrite_in_place(this.last_key, &k);
                    return Poll::Ready(Some(Ok(k)));
                }
                (None, None) => {
                    // Terminate the stream, no more items are available.
                    return Poll::Ready(None);
                }
            }
        }
    }
}
