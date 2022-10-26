use std::{fmt::Debug, pin::Pin};

use anyhow::Result;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_proto::{Message, Protobuf};

#[async_trait]
// This needs to be a trait because we want to implement it over both `State` and `StateTransaction`,
// mainly to support RPC methods.
pub trait StateRead {
    /// Get
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Gets a domain type from the State.
    async fn get<D, P>(&self, key: &str) -> Result<Option<D>>
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

    /// Gets a proto type from the State.
    async fn get_proto<D, P>(&self, key: &str) -> Result<Option<P>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let bytes = match self.get_raw(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!(e))
            .map(|v| Some(v))
    }

    /// Retrieve a raw value from non-consensus-critical ("nonconsensus") state.
    async fn get_nonconsensus(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Retrieve all values as domain types for keys matching a prefix from consensus-critical state.
    async fn prefix<D, P>(
        &self,
        prefix: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<(String, D)>> + Send + '_>>>
    where
        D: Protobuf<P>,
        P: Message + Default + 'static,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        Ok(Box::pin(self.prefix_proto(prefix).await?.map(
            |p| match p {
                Ok(p) => match D::try_from(p.1) {
                    Ok(d) => Ok((p.0, d)),
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e),
            },
        )))
    }

    /// Retrieve all values as proto types for keys matching a prefix from consensus-critical state.
    async fn prefix_proto<D, P>(
        &self,
        prefix: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<(String, P)>> + Send + '_>>>
    where
        D: Protobuf<P>,
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let o = self.prefix_raw(prefix).await?.map(|(key, bytes)| {
            Ok((
                key,
                Message::decode(&*bytes).map_err(|e| anyhow::anyhow!(e))?,
            ))
        });
        Ok(Box::pin(o))
    }

    /// Retrieve all values as raw bytes for keys matching a prefix from consensus-critical state.
    async fn prefix_raw(
        &self,
        prefix: &str,
        // TODO: it might be possible to make this zero-allocation by representing the key as a `Box<&str>` but
        // the lifetimes weren't working out, so allocating a new `String` was easier for now.
    ) -> Result<Pin<Box<dyn Stream<Item = (String, std::boxed::Box<[u8]>)> + Send + '_>>>;
}
