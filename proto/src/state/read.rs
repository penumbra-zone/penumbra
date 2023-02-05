use crate::{DomainType, Message};

use anyhow::{Context, Result};
use std::{fmt::Debug, future::Future, pin::Pin};

use async_trait::async_trait;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
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
    fn get<D>(&self, key: &str) -> Pin<Box<dyn Future<Output = Result<Option<D>>> + Send + 'static>>
    where
        D: DomainType + std::fmt::Debug,
        <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
    {
        self.get_proto(key)
            .and_then(|maybe_proto| async move {
                maybe_proto
                    .map(|proto| {
                        D::try_from(proto)
                            .map_err(Into::into)
                            .context("could not parse domain type from proto")
                    })
                    .transpose()
            })
            .boxed()
    }

    /// Gets a value from the verifiable key-value store as a proto type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a proto type `P`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a proto type `P`, or if an underlying storage error occurred.
    fn get_proto<P>(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<P>>> + Send + 'static>>
    where
        P: Message + Default + Debug,
    {
        self.get_raw(key)
            .and_then(|maybe_bytes| async move {
                match maybe_bytes {
                    None => Ok(None),
                    Some(bytes) => {
                        let v = Message::decode(&*bytes)
                            .context("could not decode proto from bytes")?;
                        Ok(Some(v))
                    }
                }
            })
            .boxed()
    }

    /// Retrieve all values for keys matching a prefix from consensus-critical state, as domain types.
    #[allow(clippy::type_complexity)]
    fn prefix<'a, D>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, D)>> + Send + 'a>>
    where
        D: DomainType,
        <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
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
    fn prefix_proto<'a, P>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, P)>> + Send + 'a>>
    where
        P: Message + Default,
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
