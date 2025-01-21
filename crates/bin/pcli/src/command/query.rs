use anyhow::{anyhow, Context, Result};

pub(crate) mod auction;
mod chain;
mod community_pool;
mod dex;
mod governance;
mod ibc_query;
mod shielded_pool;
mod tx;
mod validator;

use auction::AuctionCmd;
use base64::prelude::*;
use chain::ChainCmd;
use cnidarium::proto::v1::non_verifiable_key_value_request::Key as NVKey;
use colored_json::ToColoredJson;
use community_pool::CommunityPoolCmd;
use dex::DexCmd;
use governance::GovernanceCmd;
use ibc_query::IbcCmd;
use shielded_pool::ShieldedPool;
use tx::Tx;
pub(super) use validator::ValidatorCmd;

use crate::App;

#[derive(Clone, clap::ValueEnum, Debug)]
pub enum OutputFormat {
    Json,
    Base64,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum QueryCmd {
    /// Queries an arbitrary key.
    Key {
        /// The key to query.
        ///
        /// When querying the JMT, keys are plain string values.
        ///
        /// When querying nonverifiable storage, keys should be base64-encoded strings.
        key: String,
        /// The storage backend to query.
        ///
        /// Valid arguments are "jmt" and "nonverifiable".
        ///
        /// Defaults to the JMT.
        #[clap(long, default_value = "jmt")]
        storage_backend: String,
    },
    /// Queries shielded pool data.
    #[clap(subcommand)]
    ShieldedPool(ShieldedPool),
    /// Queries a transaction by hash.
    Tx(Tx),
    /// Queries information about the chain.
    #[clap(subcommand)]
    Chain(ChainCmd),
    /// Queries information about validators.
    #[clap(subcommand)]
    Validator(ValidatorCmd),
    /// Queries information about governance proposals.
    #[clap(subcommand)]
    Governance(GovernanceCmd),
    /// Queries information about the Community Pool.
    #[clap(subcommand)]
    CommunityPool(CommunityPoolCmd),
    /// Queries information about the decentralized exchange.
    #[clap(subcommand)]
    Dex(DexCmd),
    /// Queries information about IBC.
    #[clap(subcommand)]
    Ibc(IbcCmd),
    /// Subscribes to a filtered stream of state changes.
    Watch {
        /// The regex to filter keys in verifiable storage.
        ///
        /// The empty string matches all keys; the pattern $^ matches no keys.
        #[clap(long, default_value = "")]
        key_regex: String,
        /// The regex to filter keys in nonverifiable storage.
        ///
        /// The empty string matches all keys; the pattern $^ matches no keys.
        #[clap(long, default_value = "")]
        nv_key_regex: String,
    },
    /// Queries information about a Dutch auction.
    #[clap(subcommand)]
    Auction(AuctionCmd),
}

impl QueryCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        if let QueryCmd::Watch {
            key_regex,
            nv_key_regex,
        } = self
        {
            return watch(key_regex.clone(), nv_key_regex.clone(), app).await;
        }

        // Special-case: this is a Tendermint query
        if let QueryCmd::Tx(tx) = self {
            return tx.exec(app).await;
        }

        if let QueryCmd::Chain(chain) = self {
            return chain.exec(app).await;
        }

        if let QueryCmd::Validator(validator) = self {
            return validator.exec(app).await;
        }

        if let QueryCmd::Dex(dex) = self {
            return dex.exec(app).await;
        }

        if let QueryCmd::Governance(governance) = self {
            return governance.exec(app).await;
        }

        if let QueryCmd::CommunityPool(cp) = self {
            return cp.exec(app).await;
        }

        if let QueryCmd::Ibc(ibc) = self {
            return ibc.exec(app).await;
        }

        if let QueryCmd::Auction(auction) = self {
            return auction.exec(app).await;
        }

        // TODO: this is a hack; we should replace all raw state key uses with RPC methods.
        if let QueryCmd::ShieldedPool(ShieldedPool::CompactBlock { height }) = self {
            use penumbra_proto::core::component::compact_block::v1::{
                query_service_client::QueryServiceClient as CompactBlockQueryServiceClient,
                CompactBlockRequest,
            };
            let mut client = CompactBlockQueryServiceClient::new(app.pd_channel().await?);
            let compact_block = client
                .compact_block(CompactBlockRequest { height: *height })
                .await?
                .into_inner()
                .compact_block
                .ok_or_else(|| anyhow!("compact block missing from response"))?;
            let json = serde_json::to_string_pretty(&compact_block)?;

            println!("{}", json.to_colored_json_auto()?);
            return Ok(());
        }

        let (key, storage_backend) = match self {
            QueryCmd::Tx(_)
            | QueryCmd::Chain(_)
            | QueryCmd::Validator(_)
            | QueryCmd::Dex(_)
            | QueryCmd::Governance(_)
            | QueryCmd::CommunityPool(_)
            | QueryCmd::Watch { .. }
            | QueryCmd::Auction { .. }
            | QueryCmd::Ibc(_) => {
                unreachable!("query handled in guard");
            }
            QueryCmd::ShieldedPool(p) => (p.key().clone(), "jmt".to_string()),
            QueryCmd::Key {
                key,
                storage_backend,
            } => (key.clone(), storage_backend.clone()),
        };

        use cnidarium::proto::v1::query_service_client::QueryServiceClient;
        let mut client = QueryServiceClient::new(app.pd_channel().await?);

        // Using an enum in the clap arguments was annoying; this is workable:
        match storage_backend.as_str() {
            "nonverifiable" => {
                let key_bytes = BASE64_STANDARD
                    .decode(&key)
                    .map_err(|e| anyhow::anyhow!(format!("invalid base64: {}", e)))?;

                let req = cnidarium::proto::v1::NonVerifiableKeyValueRequest {
                    key: Some(NVKey { inner: key_bytes }),
                    ..Default::default()
                };

                tracing::debug!(?req);

                let value = client
                    .non_verifiable_key_value(req)
                    .await?
                    .into_inner()
                    .value
                    .context(format!("key not found! key={}", key))?;

                self.display_value(&value.value)?;
            }
            // Default to JMT
            "jmt" | _ => {
                let req = cnidarium::proto::v1::KeyValueRequest {
                    key: key.clone(),
                    // Command-line queries don't have a reason to include proofs as of now.
                    proof: false,
                    ..Default::default()
                };

                tracing::debug!(?req);

                let value = client
                    .key_value(req)
                    .await?
                    .into_inner()
                    .value
                    .context(format!("key not found! key={}", key))?;

                self.display_value(&value.value)?;
            }
        };

        Ok(())
    }

    pub fn offline(&self) -> bool {
        match self {
            QueryCmd::Dex { .. } | QueryCmd::CommunityPool { .. } => false,
            QueryCmd::Tx { .. }
            | QueryCmd::Chain { .. }
            | QueryCmd::Validator { .. }
            | QueryCmd::ShieldedPool { .. }
            | QueryCmd::Governance { .. }
            | QueryCmd::Key { .. }
            | QueryCmd::Watch { .. }
            | QueryCmd::Auction { .. }
            | QueryCmd::Ibc(_) => true,
        }
    }

    fn display_value(&self, bytes: &[u8]) -> Result<()> {
        match self {
            QueryCmd::Key { .. } => {
                println!("{}", hex::encode(bytes));
            }
            QueryCmd::ShieldedPool(sp) => sp.display_value(bytes)?,
            QueryCmd::Tx { .. }
            | QueryCmd::Chain { .. }
            | QueryCmd::Validator { .. }
            | QueryCmd::Dex { .. }
            | QueryCmd::Governance { .. }
            | QueryCmd::CommunityPool { .. }
            | QueryCmd::Watch { .. }
            | QueryCmd::Auction { .. }
            | QueryCmd::Ibc(_) => {
                unreachable!("query is special cased")
            }
        }

        Ok(())
    }
}

// this code (not just this function, the whole module) is pretty shitty,
// but that's maybe okay for the moment. it exists to consume the rpc.
async fn watch(key_regex: String, nv_key_regex: String, app: &mut App) -> Result<()> {
    use cnidarium::proto::v1::{query_service_client::QueryServiceClient, watch_response as wr};
    let mut client = QueryServiceClient::new(app.pd_channel().await?);

    let req = cnidarium::proto::v1::WatchRequest {
        key_regex,
        nv_key_regex,
    };

    tracing::debug!(?req);

    let mut stream = client.watch(req).await?.into_inner();

    while let Some(rsp) = stream.message().await? {
        match rsp.entry {
            Some(wr::Entry::Kv(kv)) => {
                if kv.deleted {
                    println!("{} KV {} -> DELETED", rsp.version, kv.key);
                } else {
                    println!(
                        "{} KV {} -> {}",
                        rsp.version,
                        kv.key,
                        simple_base64::encode(&kv.value)
                    );
                }
            }
            Some(wr::Entry::NvKv(nv_kv)) => {
                let key = simple_base64::encode(&nv_kv.key);

                if nv_kv.deleted {
                    println!("{} NVKV {} -> DELETED", rsp.version, key);
                } else {
                    println!(
                        "{} NVKV {} -> {}",
                        rsp.version,
                        key,
                        simple_base64::encode(&nv_kv.value)
                    );
                }
            }
            None => {
                return Err(anyhow!("server returned None event"));
            }
        }
    }

    Ok(())
}
