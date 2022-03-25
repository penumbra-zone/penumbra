use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jmt::{storage::TreeReader, KeyHash, WriteOverlay};
use penumbra_proto::{Message, Protobuf};

/// An extension trait that allows writing proto-encoded domain types to
/// a shared [`WriteOverlay`].
#[async_trait]
pub trait WriteOverlayExt {
    /// Reads a domain type from the overlay, using the proto encoding.
    async fn get_domain<D, P>(&self, key: KeyHash) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>;

    /// Puts a domain type into the overlay, using the proto encoding.
    fn put_domain<D, P>(&self, key: KeyHash, value: D)
    where
        D: Protobuf<P> + Send,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>;

    /// Reads a proto type from the overlay.
    ///
    /// It's probably preferable to use [`WriteOverlayExt::get_domain`] instead,
    /// but there are cases where it's convenient to use the proto directly.
    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default;

    /// Puts a proto type into the overlay.
    ///
    /// It's probably preferable to use [`WriteOverlayExt::put_domain`] instead,
    /// but there are cases where it's convenient to use the proto directly.
    fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message;
}

#[async_trait]
impl<R: TreeReader + Sync> WriteOverlayExt for Arc<Mutex<WriteOverlay<R>>> {
    async fn get_domain<D, P>(&self, _key: KeyHash) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        todo!()
    }

    fn put_domain<D, P>(&self, _key: KeyHash, _value: D)
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        todo!()
    }

    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default,
    {
        let s = self.lock().unwrap();
        let b = s.get(key).await?;
        drop(s);
        let bytes = match b {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        // TODO: this isn't working because the impl for decode requires a
        // Default impl for `P`
        Message::decode(bytes.into())
            .map_err(|e| anyhow!(e))
            .map(|v| Some(v))
    }

    fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message,
    {
        self.lock().unwrap().put(key, value.encode_to_vec());
    }
}
