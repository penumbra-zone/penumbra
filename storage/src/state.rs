use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use jmt::{KeyHash, WriteOverlay};
use penumbra_proto::Protobuf;
use prost::Message;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{StateExt, Storage};

/// A lightweight, copy-on-write fork of the application state.
///
/// A `State` instance is created by [`Storage::state`], which
/// forks off of a specific version of the underlying `Storage`.  The `State`
/// allows both read and write access, but as a copy-on-write fork, writes are
/// stored in-memory until [`State::commit`] is called; if it is never called,
/// they are discarded.  Reads from the `State` return uncommitted writes, if
/// any.
///
/// The `State` can be cheaply cloned, and uses an async [`RwLock`] internally
/// to allow shared access.  Note that cloning an existing `State` instance is
/// **different** from creating a new `State` from the [`Storage`]: cloning an
/// existing instance shares the same copy-on-write fork, while creating a new
/// instance creates a new fork.
#[derive(Clone)]
pub struct State {
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    overlay: WriteOverlay<Storage>,
}

impl State {
    /// This is pub(crate) because people should use [`Storage::state`] instead.
    pub(crate) async fn new(storage: Storage) -> anyhow::Result<Self> {
        // If the tree is empty, use PRE_GENESIS_VERSION as the version,
        // so that the first commit will be at version 0.
        let version = storage
            .latest_version()
            .await?
            .unwrap_or(WriteOverlay::<Storage>::PRE_GENESIS_VERSION);
        tracing::debug!("creating state for version {}", version);

        let inner = Inner {
            overlay: WriteOverlay::new(storage, version),
        };

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
        })
    }
}

#[async_trait]
impl StateExt for State {
    #[instrument(skip(self, key))]
    async fn get_domain<D, P>(&self, key: KeyHash) -> anyhow::Result<Option<D>>
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
            Err(e) => Err(e),
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
    async fn get_proto<P>(&self, key: KeyHash) -> anyhow::Result<Option<P>>
    where
        P: Message + Default + Debug,
    {
        let bytes = match self.inner.read().await.overlay.get(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!(e))
            .map(|v| Some(v))
    }

    #[instrument(skip(self, key, value))]
    async fn put_proto<P>(&self, key: KeyHash, value: P)
    where
        P: Message + Debug,
    {
        self.inner
            .write()
            .await
            .overlay
            .put(key, value.encode_to_vec());
    }
}
