//! State read and write extensions for token factory.

// prost::Message provides encode_to_vec()/decode(); without it this module
// does not compile under the `component` feature.
use prost::Message as _;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_asset::asset;

use crate::{error::TokenFactoryError, TokenFactoryId, TokenFactoryParameters};

use super::state_key;

/// Extension trait for reading token factory state.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Check if a token factory nonce has already been used.
    async fn token_factory_nonce_used(&self, id: &TokenFactoryId) -> Result<bool> {
        Ok(self.get_raw(&state_key::nonce_used(id)).await?.is_some())
    }

    /// Get token metadata by ID.
    async fn token_factory_metadata(&self, id: &TokenFactoryId) -> Result<Option<asset::Metadata>> {
        let key = state_key::token_metadata(id);
        let Some(bytes) = self.get_raw(&key).await? else {
            return Ok(None);
        };
        let proto =
            penumbra_sdk_proto::core::asset::v1::Metadata::decode(bytes.as_slice())?;
        Ok(Some(asset::Metadata::try_from(proto)?))
    }

    /// Read the governance-controlled token factory parameters.
    ///
    /// Absent keys read as `false`, so a chain that has never set them has the
    /// token factory fully disabled (dormant) — the intended launch state.
    async fn token_factory_parameters(&self) -> Result<TokenFactoryParameters> {
        let token_factory_enabled = matches!(
            self.get_raw(state_key::param_enabled()).await?,
            Some(b) if b.first() == Some(&1u8)
        );
        let mintable_enabled = matches!(
            self.get_raw(state_key::param_mintable_enabled()).await?,
            Some(b) if b.first() == Some(&1u8)
        );
        Ok(TokenFactoryParameters {
            token_factory_enabled,
            mintable_enabled,
        })
    }
}

impl<T> StateReadExt for T where T: StateRead + ?Sized {}

/// Extension trait for writing token factory state.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Mark a token factory nonce as used.
    ///
    /// # Errors
    ///
    /// Returns an error if the nonce has already been used.
    async fn token_factory_mark_nonce_used(&mut self, id: &TokenFactoryId) -> Result<()> {
        let key = state_key::nonce_used(id);

        // Check if already used
        if self.get_raw(&key).await?.is_some() {
            return Err(TokenFactoryError::NonceAlreadyUsed.into());
        }

        // Mark as used by writing an empty value
        self.put_raw(key, vec![1u8]);
        Ok(())
    }

    /// Store token metadata for a factory token.
    fn token_factory_put_metadata(&mut self, id: &TokenFactoryId, metadata: &asset::Metadata) {
        let key = state_key::token_metadata(id);
        let proto: penumbra_sdk_proto::core::asset::v1::Metadata = metadata.clone().into();
        self.put_raw(key, proto.encode_to_vec());
    }

    /// Write the governance-controlled token factory parameters.
    fn put_token_factory_parameters(&mut self, params: &TokenFactoryParameters) {
        self.put_raw(
            state_key::param_enabled().to_string(),
            vec![params.token_factory_enabled as u8],
        );
        self.put_raw(
            state_key::param_mintable_enabled().to_string(),
            vec![params.mintable_enabled as u8],
        );
    }
}

impl<T> StateWriteExt for T where T: StateWrite + ?Sized {}
