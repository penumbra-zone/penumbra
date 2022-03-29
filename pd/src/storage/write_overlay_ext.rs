use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jmt::{storage::TreeReader, KeyHash, WriteOverlay};
use penumbra_proto::{Message, Protobuf};
use tokio::sync::Mutex;

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
    async fn put_domain<D, P>(&self, key: KeyHash, value: D)
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
    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message;
}

#[async_trait]
impl<R: TreeReader + Sync> WriteOverlayExt for Arc<Mutex<WriteOverlay<R>>> {
    async fn get_domain<D, P>(&self, key: KeyHash) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        match self.get_proto(key).await {
            Ok(Some(p)) => match D::try_from(p) {
                Ok(d) => Ok(Some(d)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn put_domain<D, P>(&self, key: KeyHash, value: D)
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        D: std::marker::Send,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let v: P = value.try_into().unwrap();
        self.put_proto(key, v).await;
    }

    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default,
    {
        let bytes = match self.lock().await.get(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow!(e))
            .map(|v| Some(v))
    }

    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message,
    {
        self.lock().await.put(key, value.encode_to_vec());
    }
}
