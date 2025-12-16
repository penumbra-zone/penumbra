//! State read and write extensions for token factory.

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};

use crate::{error::TokenFactoryError, TokenFactoryId};

use super::state_key;

/// Extension trait for reading token factory state.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Check if a token factory nonce has already been used.
    async fn token_factory_nonce_used(&self, id: &TokenFactoryId) -> Result<bool> {
        Ok(self.get_raw(&state_key::nonce_used(id)).await?.is_some())
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
}

impl<T> StateWriteExt for T where T: StateWrite + ?Sized {}
