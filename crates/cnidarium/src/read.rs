use std::{any::Any, future::Future, ops::RangeBounds, sync::Arc};

use anyhow::Result;
use futures::Stream;

/// Read access to chain state.
pub trait StateRead: Send + Sync {
    type GetRawFut: Future<Output = Result<Option<Vec<u8>>>> + Send + 'static;
    type PrefixRawStream: Stream<Item = Result<(String, Vec<u8>)>> + Send + 'static;
    type PrefixKeysStream: Stream<Item = Result<String>> + Send + 'static;
    type NonconsensusPrefixRawStream: Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'static;
    type NonconsensusRangeRawStream: Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'static;

    /// Gets a value from the verifiable key-value store as raw bytes.
    ///
    /// Users should generally prefer to use `get` or `get_proto` from an extension trait.
    fn get_raw(&self, key: &str) -> Self::GetRawFut;

    /// Gets a byte value from the non-verifiable key-value store.
    ///
    /// This is intended for application-specific indexes of the verifiable
    /// consensus state, rather than for use as a primary data storage method.
    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut;

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
    ///
    /// # Panics
    ///
    /// If there *is* a value at `key` but it is not of the type requested.
    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T>;

    /// Gets the [`TypeId`] of the object stored at `key` in the ephemeral key-object store, if any
    /// is present.
    fn object_type(&self, key: &'static str) -> Option<std::any::TypeId>;

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use `prefix` or `prefix_proto` from an extension trait.
    fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream;

    /// Retrieve all keys (but not values) matching a prefix from the verifiable key-value store.
    fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream;

    /// Retrieve all values for keys matching a prefix from the non-verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use wrapper methods in an extension trait.
    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream;

    /// Retrieve all values for keys in a range from the non-verifiable key-value store, as raw bytes.
    /// This method does not support inclusive ranges, and will return an error if passed one.
    ///
    /// Users should generally prefer to use wrapper methods in an extension trait.
    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl RangeBounds<Vec<u8>>,
    ) -> Result<Self::NonconsensusRangeRawStream>;
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a S {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;
    type NonconsensusRangeRawStream = S::NonconsensusRangeRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        (**self).prefix_keys(prefix)
    }

    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        (**self).nonverifiable_prefix_raw(prefix)
    }

    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl std::ops::RangeBounds<Vec<u8>>,
    ) -> anyhow::Result<Self::NonconsensusRangeRawStream> {
        (**self).nonverifiable_range_raw(prefix, range)
    }

    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonverifiable_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        (**self).object_get(key)
    }

    fn object_type(&self, key: &'static str) -> Option<std::any::TypeId> {
        (**self).object_type(key)
    }
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a mut S {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;
    type NonconsensusRangeRawStream = S::NonconsensusRangeRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        (**self).prefix_keys(prefix)
    }

    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        (**self).nonverifiable_prefix_raw(prefix)
    }

    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl RangeBounds<Vec<u8>>,
    ) -> Result<S::NonconsensusRangeRawStream> {
        (**self).nonverifiable_range_raw(prefix, range)
    }

    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonverifiable_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        (**self).object_get(key)
    }

    fn object_type(&self, key: &'static str) -> Option<std::any::TypeId> {
        (**self).object_type(key)
    }
}

impl<S: StateRead + Send + Sync> StateRead for Arc<S> {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;
    type NonconsensusRangeRawStream = S::NonconsensusRangeRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        (**self).prefix_keys(prefix)
    }

    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        (**self).nonverifiable_prefix_raw(prefix)
    }

    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl RangeBounds<Vec<u8>>,
    ) -> Result<Self::NonconsensusRangeRawStream> {
        (**self).nonverifiable_range_raw(prefix, range)
    }

    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonverifiable_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        (**self).object_get(key)
    }

    fn object_type(&self, key: &'static str) -> Option<std::any::TypeId> {
        (**self).object_type(key)
    }
}

impl StateRead for () {
    type GetRawFut = futures::future::Ready<Result<Option<Vec<u8>>>>;
    type PrefixRawStream = futures::stream::Iter<std::iter::Empty<Result<(String, Vec<u8>)>>>;
    type PrefixKeysStream = futures::stream::Iter<std::iter::Empty<Result<String>>>;
    type NonconsensusPrefixRawStream =
        futures::stream::Iter<std::iter::Empty<Result<(Vec<u8>, Vec<u8>)>>>;
    type NonconsensusRangeRawStream =
        futures::stream::Iter<std::iter::Empty<Result<(Vec<u8>, Vec<u8>)>>>;

    fn get_raw(&self, _key: &str) -> Self::GetRawFut {
        futures::future::ready(Ok(None))
    }

    fn nonverifiable_get_raw(&self, _key: &[u8]) -> Self::GetRawFut {
        futures::future::ready(Ok(None))
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, _key: &'static str) -> Option<T> {
        None
    }

    fn object_type(&self, _key: &'static str) -> Option<std::any::TypeId> {
        None
    }

    fn prefix_raw(&self, _prefix: &str) -> Self::PrefixRawStream {
        futures::stream::iter(std::iter::empty())
    }

    fn prefix_keys(&self, _prefix: &str) -> Self::PrefixKeysStream {
        futures::stream::iter(std::iter::empty())
    }

    fn nonverifiable_prefix_raw(&self, _prefix: &[u8]) -> Self::NonconsensusPrefixRawStream {
        futures::stream::iter(std::iter::empty())
    }

    fn nonverifiable_range_raw(
        &self,
        _prefix: Option<&[u8]>,
        _range: impl RangeBounds<Vec<u8>>,
    ) -> Result<Self::NonconsensusRangeRawStream> {
        Ok(futures::stream::iter(std::iter::empty()))
    }
}
