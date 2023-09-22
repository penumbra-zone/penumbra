use anyhow::Result;
use penumbra_keys::Address;
use penumbra_proto::tools::summoning::v1alpha1::CeremonyCrs;

#[derive(Clone)]
pub struct Storage {
    // TODO: This should have methods for persisting all of the state of the coordinator,
    // using a sqlite database.
}

impl Storage {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn can_contribute(&self, address: Address) -> Result<()> {
        // Criteria:
        // - Not banned
        // - Bid more than min amount
        // - Hasn't already contributed
        Ok(())
    }

    pub async fn current_crs(&self) -> Result<CeremonyCrs> {
        Ok(CeremonyCrs::default())
    }

    // TODO: Add other stuff here
    pub async fn commit_contribution(&self, contributor: Address, crs: &CeremonyCrs) -> Result<()> {
        // TODO: Do.
        Ok(())
    }
}
