use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;

use penumbra_proto::{Message, Protobuf};

#[async_trait]
pub trait StateRead {
    /// Get
    fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Gets a domain type from the State.
    fn get<D, P>(&self, key: &str) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        match self.get_proto(key) {
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
    fn get_proto<D, P>(&self, key: &str) -> Result<Option<P>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let bytes = match self.get_raw(key)? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!(e))
            .map(|v| Some(v))
    }

    /// Retrieve a value from non-consensus-critical ("sidecar") state.
    fn get_sidecar(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
