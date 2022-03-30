use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jmt::{storage::TreeReader, KeyHash, WriteOverlay};
use penumbra_proto::{Message, Protobuf};
use std::fmt::Debug;
use std::sync::Arc;
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
        D: Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>;

    /// Puts a domain type into the overlay, using the proto encoding.
    async fn put_domain<D, P>(&self, key: KeyHash, value: D)
    where
        D: Protobuf<P> + Send,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone,
        D: Debug,
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
        D: Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        tracing::debug!(keyhash = ?key, "Requested KeyHash");

        match self.get_proto(key).await {
            Ok(Some(p)) => match D::try_from(p) {
                Ok(d) => {
                    tracing::debug!(domain = ?d, "Retrieved domain");

                    Ok(Some(d))
                }
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
        tracing::debug!(keyhash = ?key, "Inserted KeyHash");

        let v: P = value.try_into().unwrap();

        tracing::debug!(val = ?v, "Inserted value");

        self.put_proto(key, v).await;
    }

    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default,
    {
        tracing::debug!(keyhash = ?key, "Requested KeyHash");

        let bytes = match self.lock().await.get(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow!(e))
            .map(|v| {
                tracing::debug!(val = ?v, "Retrieved proto");
                Some(v)
            })
    }

    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message,
    {
        tracing::debug!(keyhash = ?key, "Inserted KeyHash");

        tracing::debug!(val = ?value, "Inserted value");

        self.lock().await.put(key, value.encode_to_vec());
    }
}
