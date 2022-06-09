use anyhow::Result;
use jmt::KeyHash;
use penumbra_proto::Protobuf;

use crate::Opt;

#[derive(Debug, clap::Subcommand)]
pub enum QueryCmd {
    /// Queries an arbitrary key.
    Key {
        /// The key to query.
        key: String,
    },
    /// Queries shielded pool data.
    #[clap(subcommand)]
    ShieldedPool(ShieldedPool),
}

#[derive(Debug, clap::Subcommand)]
pub enum ShieldedPool {
    /// Queries the note commitment tree anchor for a given height.
    Anchor {
        /// The height to query.
        height: u64,
    },
}

impl QueryCmd {
    pub async fn exec(&self, opt: &Opt) -> Result<()> {
        let mut client = opt.specific_client().await?;

        let key_hash = self.key_hash();
        tracing::debug!(?key_hash);

        let rsp = client
            .key_value(penumbra_proto::client::specific::KeyValueRequest {
                key_hash: key_hash.0.to_vec(),
                ..Default::default()
            })
            .await?
            .into_inner();

        self.display_value(&rsp.value)?;
        Ok(())
    }

    fn key_hash(&self) -> KeyHash {
        match self {
            QueryCmd::Key { key } => key.as_bytes().into(),
            QueryCmd::ShieldedPool(sp) => sp.key_hash(),
        }
    }

    fn display_value(&self, bytes: &[u8]) -> Result<()> {
        match self {
            QueryCmd::Key { .. } => {
                println!("{}", hex::encode(bytes));
            }
            QueryCmd::ShieldedPool(sp) => sp.display_value(bytes)?,
        }

        Ok(())
    }
}

impl ShieldedPool {
    fn key_hash(&self) -> KeyHash {
        use penumbra_component::shielded_pool::state_key;
        match self {
            ShieldedPool::Anchor { height } => state_key::anchor_by_height(height),
        }
    }

    fn display_value(&self, bytes: &[u8]) -> Result<()> {
        match self {
            ShieldedPool::Anchor { .. } => {
                let anchor = penumbra_tct::Root::decode(bytes)?;
                println!("{:#?}", anchor);
            }
        }
        Ok(())
    }
}
