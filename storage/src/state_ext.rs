use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use jmt::KeyHash;
use penumbra_proto::{Message, Protobuf};

/// An extension trait that allows writing proto-encoded domain types to a
/// shared [`State`](crate::State).
///
/// Writing these methods as a trait allows different parts of the application
/// to describe how their data is recorded in the state using "view traits" of
/// the form `FooView: StateExt`.
///
/// (If the methods were on the `State` directly, this would be more cumbersome,
/// since it would no longer be possible to write the `FooView` trait with
/// provided methods, forcing duplicate declarations of every method in the
/// extension trait impl as well as in the definition).
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
