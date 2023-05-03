use crate::{DomainType, Message};

use anyhow::Result;
use std::{fmt::Debug, pin::Pin};

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_storage::StateRead;

use super::future::{DomainFuture, ProtoFuture};

#[async_trait]
pub trait StateReadProto: StateRead + Send + Sync {
    /// Gets a value from the verifiable key-value store as a domain type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a domain type `D`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a domain type `D`, or if an underlying storage error occurred.
    fn get<D>(&self, key: &str) -> DomainFuture<D, Self::GetRawFut>
    where
        D: DomainType + std::fmt::Debug,
        <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
    {
        DomainFuture {
            inner: self.get_raw(key),
            _marker: std::marker::PhantomData,
        }
    }

    /// Gets a value from the verifiable key-value store as a proto type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a proto type `P`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a proto type `P`, or if an underlying storage error occurred.
    fn get_proto<P>(&self, key: &str) -> ProtoFuture<P, Self::GetRawFut>
    where
        P: Message + Default + Debug,
    {
        ProtoFuture {
            inner: self.get_raw(key),
            _marker: std::marker::PhantomData,
        }
    }

    /// Retrieve all values for keys matching a prefix from consensus-critical state, as domain types.
    #[allow(clippy::type_complexity)]
    fn prefix<'a, D>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, D)>> + Send + 'static>>
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
    ) -> Pin<Box<dyn Stream<Item = Result<(String, P)>> + Send + 'static>>
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
