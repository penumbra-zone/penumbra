use crate::{Message, Protobuf};

use anyhow::Result;
use std::fmt::Debug;
use std::pin::Pin;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_storage::StateRead;

#[async_trait]
pub trait StateReadProto: StateRead + Send + Sync {
    /// Gets a value from the verifiable key-value store as a domain type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a domain type `D`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a domain type `D`, or if an underlying storage error occurred.
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

    /// Gets a value from the verifiable key-value store as a proto type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a proto type `P`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a proto type `P`, or if an underlying storage error occurred.
    async fn get_proto<P>(&self, key: &str) -> Result<Option<P>>
    where
        P: Message + Default + Debug,
    {
        let bytes = match self.get_raw(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!(e))
            .map(|v| Some(v))
    }

    /// Retrieve all values for keys matching a prefix from consensus-critical state, as domain types.
    #[allow(clippy::type_complexity)]
    fn prefix<'a, D, P>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, D)>> + Send + 'a>>
    where
        D: Protobuf<P>,
        P: Message + Default + 'static,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        Box::pin(self.prefix_proto(prefix).map(|p| match p {
            Ok(p) => match D::try_from(p.1) {
                Ok(d) => Ok((p.0, d)),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e),
        }))
    }

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as proto types.
    #[allow(clippy::type_complexity)]
    fn prefix_proto<'a, D, P>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, P)>> + Send + 'a>>
    where
        D: Protobuf<P>,
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let o = self.prefix_raw(prefix).map(|r| {
            r.and_then(|(key, bytes)| {
                Ok((
                    key,
                    Message::decode(&*bytes).map_err(|e| anyhow::anyhow!(e))?,
                ))
            })
        });
        Box::pin(o)
    }
}
impl<T: StateRead + ?Sized> StateReadProto for T {}
