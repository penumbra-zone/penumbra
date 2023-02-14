use std::{any::Any, future::Future};

use anyhow::Result;
use futures::Stream;

/// Read access to chain state.
pub trait StateRead: Send + Sync {
    type GetRawFut: Future<Output = Result<Option<Vec<u8>>>> + Send + 'static;
    type PrefixRawStream: Stream<Item = Result<(String, Vec<u8>)>> + Send + 'static;
    type PrefixKeysStream: Stream<Item = Result<String>> + Send + 'static;
    type NonconsensusPrefixRawStream: Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'static;

    /// Gets a value from the verifiable key-value store as raw bytes.
    ///
    /// Users should generally prefer to use `get` or `get_proto` from an extension trait.
    fn get_raw(&self, key: &str) -> Self::GetRawFut;

    /// Gets a byte value from the non-verifiable key-value store.
    ///
    /// This is intended for application-specific indexes of the verifiable
    /// consensus state, rather than for use as a primary data storage method.
    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut;

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
    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T>;

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use `prefix` or `prefix_proto` from an extension trait.
    fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream;

    /// Retrieve all keys (but not values) matching a prefix from the verifiable key-value store.
    fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream;

    /// Retrieve all values for keys matching a prefix from the non-verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use wrapper methods in an extension trait.
    fn nonconsensus_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream;
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a S {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        (**self).object_get(key)
    }
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a mut S {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        (**self).object_get(key)
    }
}
