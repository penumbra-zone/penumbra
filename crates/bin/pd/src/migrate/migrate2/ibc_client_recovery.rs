use anyhow::Result;
use cnidarium::StateDelta;
use ibc_types::core::client::ClientId;
use penumbra_sdk_app::PenumbraHost;
use penumbra_sdk_ibc::component::ClientRecoveryExt;

use super::framework::Migration;

pub struct IbcClientRecoveryMigration {
    old_client_id: String,
    new_client_id: String,
    app_version: Option<u64>,
}

impl IbcClientRecoveryMigration {
    pub fn new(old_client_id: String, new_client_id: String, app_version: Option<u64>) -> Self {
        Self {
            old_client_id,
            new_client_id,
            app_version,
        }
    }
}

impl Migration for IbcClientRecoveryMigration {
    fn name(&self) -> &'static str {
        "ibc-client-recovery"
    }

    fn target_app_version(&self) -> Option<u64> {
        self.app_version
    }

    async fn migrate_inner(&self, delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        tracing::info!(
            old_client_id = %self.old_client_id,
            new_client_id = %self.new_client_id,
            "performing IBC client recovery migration"
        );

        // Parse the client IDs
        let subject_client_id: ClientId = self
            .old_client_id
            .parse()
            .map_err(|e| anyhow::format_err!("invalid old client ID: {}", e))?;
        let substitute_client_id: ClientId = self
            .new_client_id
            .parse()
            .map_err(|e| anyhow::format_err!("invalid new client ID: {}", e))?;

        // Use the ClientRecoveryExt trait to perform the recovery
        // All validation logic is encapsulated in the trait method
        delta
            .recover_client::<PenumbraHost>(&subject_client_id, &substitute_client_id)
            .await?;

        Ok(())
    }
}
