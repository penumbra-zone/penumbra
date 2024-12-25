// These mock-consensus helper traits aren't consumed just yet.
#![allow(dead_code)]

use {
    async_trait::async_trait,
    futures::TryStreamExt,
    penumbra_sdk_proto::StateReadProto,
    penumbra_sdk_stake::{
        component::validator_handler::ValidatorDataRead, state_key, validator::Validator,
        IdentityKey,
    },
};

/// All [`ValidatorDataRead`]s implement [`ValidatorDataReadExt`].
impl<T: ValidatorDataRead + ?Sized> ValidatorDataReadExt for T {}

/// Additional extensions to [`ValidatorDataRead`] for use in test cases.
#[async_trait]
pub trait ValidatorDataReadExt: ValidatorDataRead {
    /// Returns a list of **all** known validators' metadata.
    ///
    /// This is not included in [`ValidatorDataRead`] because it is liable to become expensive
    /// over time as more validators are defined. This should only be used in test cases.
    async fn validator_definitions(&self) -> anyhow::Result<Vec<Validator>> {
        self.prefix(state_key::validators::definitions::prefix())
            .map_ok(|(_key, validator)| validator)
            .try_collect()
            .await
    }

    /// Returns a list of **all** known validators' identity keys.
    ///
    /// This is not included in [`ValidatorDataRead`] because it is liable to become expensive
    /// over time as more validators are defined. This should only be used in test cases.
    async fn validator_identity_keys(&self) -> anyhow::Result<Vec<IdentityKey>> {
        self.prefix(state_key::validators::definitions::prefix())
            .map_ok(|(_key, validator)| validator)
            .map_ok(|validator: Validator| validator.identity_key)
            .try_collect()
            .await
    }
}
