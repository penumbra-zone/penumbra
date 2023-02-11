use std::{any::Any, future::Future, pin::Pin};

use anyhow::Result;
use futures::Stream;

/// Read access to chain state.
pub trait StateRead: Send + Sync {
    type GetRawFut: Future<Output = Result<Option<Vec<u8>>>> + Send + 'static;

    /// Gets a value from the verifiable key-value store as raw bytes.
    ///
    /// Users should generally prefer to use `get` or `get_proto` from an extension trait.
    fn get_raw(&self, key: &str) -> Self::GetRawFut;

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use `prefix` or `prefix_proto` from an extension trait.
    #[allow(clippy::type_complexity)]
    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>>;

    /// Retrieve all keys (but not values) matching a prefix from the verifiable key-value store.
    fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>>;

    /// Gets a byte value from the non-verifiable key-value store.
    ///
    /// This is intended for application-specific indexes of the verifiable
    /// consensus state, rather than for use as a primary data storage method.
    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut;

    /// Retrieve all values for keys matching a prefix from the non-verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use wrapper methods in an extension trait.
    #[allow(clippy::type_complexity)]
    fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>>;

    /// Gets an object from the ephemeral key-object store.
    ///
    /// This is intended to allow application components to build up batched
    /// data transactionally, ensuring that a transaction's contributions to
    /// some batched data are only included if the entire transaction executed
    /// successfully.  This data is not persisted to the `Storage` during
    /// `commit`.
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if a value of type `T` was present at `key`.
    /// - `None` if `key` was not present, or if `key` was present but the value was not of type `T`.
    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T>;
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a S {
    type GetRawFut = S::GetRawFut;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'b>> {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'b>> {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw<'b>(
        &'b self,
        prefix: &'b [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'b>> {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        (**self).object_get(key)
    }
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a mut S {
    type GetRawFut = S::GetRawFut;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'b>> {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'b>> {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw<'b>(
        &'b self,
        prefix: &'b [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'b>> {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        (**self).object_get(key)
    }
}
