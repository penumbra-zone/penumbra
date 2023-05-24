#[macro_use]
extern crate tracing;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use penumbra_chain::params::ChainParameters;
use penumbra_compact_block::CompactBlock;
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_service_client::ObliviousQueryServiceClient, ChainParametersRequest,
        CompactBlockRangeRequest,
    },
    Message,
};
use url::Url;

#[derive(Debug, Parser)]
#[clap(
    name = "penumbra-measure",
    about = "A developer tool for measuring things about Penumbra.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The URL for the gRPC endpoint of the remote pd node.
    #[clap(
        short,
        long,
        default_value = "http://testnet.penumbra.zone:8080",
        env = "PENUMBRA_NODE_PD_URL",
        parse(try_from_str = url::Url::parse)
    )]
    node: Url,
    // TODO: use TendermintProxyService instead
    /// The URL for the Tendermint RPC endpoint of the remote node.
    #[clap(
        long,
        default_value = "http://testnet.penumbra.zone:26657",
        env = "PENUMBRA_NODE_TM_URL"
    )]
    tendermint_url: Url,
    #[clap(subcommand)]
    pub cmd: Command,
    /// The filter for log messages.
    #[clap( long, default_value_t = EnvFilter::new("warn"), env = "RUST_LOG")]
    trace_filter: EnvFilter,
}

impl Opt {
    pub fn init_tracing(&mut self) {
        tracing_subscriber::fmt()
            .with_env_filter(std::mem::take(&mut self.trace_filter))
            .init();
    }
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Measure the performance of downloading compact blocks without parsing them.
    StreamBlocks {
        /// If set, skip downloading the genesis compact block.
        #[clap(long)]
        skip_genesis: bool,
    },
}

impl Opt {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.cmd {
            Command::StreamBlocks { skip_genesis } => {
                let mut client =
                    ObliviousQueryServiceClient::connect(self.node.to_string()).await?;

                let params: ChainParameters = client
                    .chain_parameters(tonic::Request::new(ChainParametersRequest {
                        chain_id: String::new(),
                    }))
                    .await?
                    .into_inner()
                    .try_into()?;

                let end_height = self.latest_known_block_height().await?.0;
                let start_height = if skip_genesis { 1 } else { 0 };

                let mut stream = client
                    .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
                        chain_id: params.chain_id,
                        start_height,
                        end_height,
                        keep_alive: false,
                    }))
                    .await?
                    .into_inner();

                use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
                let progress_bar =
                    ProgressBar::with_draw_target(end_height, ProgressDrawTarget::stderr())
                        .with_style(ProgressStyle::default_bar().template(
                            "[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}",
                        ));
                progress_bar.set_position(0);

                let mut bytes = 0;
                let mut cb_count = 0;
                let mut nf_count = 0;
                let mut sp_rolled_up_count = 0;
                let mut sp_note_count = 0;
                let mut sp_swap_count = 0;

                use penumbra_compact_block::StatePayload;

                while let Some(block_rsp) = stream.message().await? {
                    cb_count += 1;
                    bytes += block_rsp.encoded_len();
                    let block: CompactBlock = block_rsp.try_into()?;
                    nf_count += block.nullifiers.len();
                    sp_rolled_up_count += block
                        .state_payloads
                        .iter()
                        .filter(|sp| matches!(sp, StatePayload::RolledUp { .. }))
                        .count();
                    sp_note_count += block
                        .state_payloads
                        .iter()
                        .filter(|sp| matches!(sp, StatePayload::Note { .. }))
                        .count();
                    sp_swap_count += block
                        .state_payloads
                        .iter()
                        .filter(|sp| matches!(sp, StatePayload::Swap { .. }))
                        .count();
                    progress_bar.set_position(block.height);
                }
                progress_bar.finish();

                let sp_count = sp_note_count + sp_swap_count + sp_rolled_up_count;
                println!(
                    "Fetched at least {}",
                    bytesize::to_string(bytes as u64, false)
                );
                println!("Fetched {cb_count} compact blocks, containing:");
                println!("\t{nf_count} nullifiers");
                println!("\t{sp_count} state payloads, containing:");
                println!("\t\t{sp_note_count} note payloads");
                println!("\t\t{sp_swap_count} swap payloads");
                println!("\t\t{sp_rolled_up_count} rolled up payloads");
            }
        }

        Ok(())
    }

    // This code is ripped from the view service code, and could be split out into something common.
    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> Result<(u64, bool), anyhow::Error> {
        let client = reqwest::Client::new();

        let rsp: serde_json::Value = client
            .get(format!("{}/status", self.tendermint_url.clone()))
            .send()
            .await?
            .json()
            .await?;

        tracing::debug!("{}", rsp);

        let sync_info = rsp
            .get("result")
            .and_then(|r| r.get("sync_info"))
            .ok_or_else(|| anyhow::anyhow!("could not parse sync_info in JSON response"))?;

        let latest_block_height = sync_info
            .get("latest_block_height")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("could not parse latest_block_height in JSON response"))?
            .parse()?;

        let node_catching_up = sync_info
            .get("catching_up")
            .and_then(|c| c.as_bool())
            .ok_or_else(|| anyhow::anyhow!("could not parse catching_up in JSON response"))?;

        // There is a `max_peer_block_height` available in TM 0.35, however it should not be used
        // as it does not seem to reflect the consensus height. Since clients use `latest_known_block_height`
        // to determine the height to attempt syncing to, a validator reporting a non-consensus height
        // can cause a DoS to clients attempting to sync if `max_peer_block_height` is used.
        let latest_known_block_height = latest_block_height;

        tracing::debug!(
            ?latest_block_height,
            ?node_catching_up,
            ?latest_known_block_height
        );

        Ok((latest_known_block_height, node_catching_up))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut opt = Opt::parse();
    opt.init_tracing();
    opt.run().await?;
    Ok(())
}
