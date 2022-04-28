use std::{fmt::Debug, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use jmt::{storage::TreeReader, KeyHash, WriteOverlay};
use penumbra_proto::{Message, Protobuf};
use tokio::sync::Mutex;
use tracing::instrument;

/// An extension trait that allows writing proto-encoded domain types to
/// a shared [`State`].
#[async_trait]
pub trait StateExt: Send + Sync + Sized + Clone + 'static {
    /// Reads a domain type from the state, using the proto encoding.
    async fn get_domain<D, P>(&self, key: KeyHash) -> Result<Option<D>>
    where
        D: Protobuf<P> + TryFrom<P> + Clone + Debug,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default + From<D>,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>;

    /// Puts a domain type into the state, using the proto encoding.
    async fn put_domain<D, P>(&self, key: KeyHash, value: D)
    where
        D: Protobuf<P> + Send + TryFrom<P> + Clone + Debug,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default + From<D>,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>;

    /// Reads a proto type from the state.
    ///
    /// It's probably preferable to use [`StateExt::get_domain`] instead,
    /// but there are cases where it's convenient to use the proto directly.
    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default + Debug;

    /// Puts a proto type into the state.
    ///
    /// It's probably preferable to use [`StateExt::put_domain`] instead,
    /// but there are cases where it's convenient to use the proto directly.
    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message + Debug;
}

#[async_trait]
impl<R: TreeReader + Sync + 'static> StateExt for Arc<Mutex<WriteOverlay<R>>> {
    #[instrument(skip(self, key))]
    async fn get_domain<D, P>(&self, key: KeyHash) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        match self.get_proto(key).await {
            Ok(Some(p)) => match D::try_from(p) {
                Ok(d) => {
                    tracing::trace!(?key, value = ?d);
                    Ok(Some(d))
                }
                Err(e) => Err(e.into()),
            },
            Ok(None) => {
                tracing::trace!(?key, "no entry in tree");
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(self, key, value))]
    async fn put_domain<D, P>(&self, key: KeyHash, value: D)
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Send + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        tracing::trace!(?key, ?value);
        self.put_proto(key, P::from(value)).await;
    }

    #[instrument(skip(self, key))]
    async fn get_proto<P>(&self, key: KeyHash) -> Result<Option<P>>
    where
        P: Message + Default + Debug,
    {
        let bytes = match self.lock().await.get(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow!(e))
            .map(|v| Some(v))
    }

    #[instrument(skip(self, key, value))]
    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message + Debug,
    {
        self.lock().await.put(key, value.encode_to_vec());
    }
}
