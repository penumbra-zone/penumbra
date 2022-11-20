#[macro_use]
extern crate tracing;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use penumbra_chain::{params::ChainParameters, sync::CompactBlock};
use penumbra_proto::client::v1alpha1::{
    oblivious_query_service_client::ObliviousQueryServiceClient, ChainParametersRequest,
    CompactBlockRangeRequest,
};

#[derive(Debug, Parser)]
#[clap(
    name = "penumbra-measure",
    about = "A developer tool for measuring things about Penumbra.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The hostname of the pd+tendermint node.
    #[clap(
        short,
        long,
        default_value = "testnet.penumbra.zone",
        env = "PENUMBRA_NODE_HOSTNAME",
        parse(try_from_str = url::Host::parse)
    )]
    node: url::Host,
    /// The port to use to speak to tendermint's RPC server.
    #[clap(long, default_value_t = 26657, env = "PENUMBRA_TENDERMINT_PORT")]
    tendermint_port: u16,
    /// The port to use to speak to pd's gRPC server.
    #[clap(long, default_value_t = 8080, env = "PENUMBRA_PD_PORT")]
    pd_port: u16,
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
    StreamBlocks,
}

impl Opt {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.cmd {
            Command::StreamBlocks => {
                let mut client = ObliviousQueryServiceClient::connect(format!(
                    "http://{}:{}",
                    self.node, self.pd_port
                ))
                .await?;

                let params: ChainParameters = client
                    .chain_parameters(tonic::Request::new(ChainParametersRequest {
                        chain_id: String::new(),
                    }))
                    .await?
                    .into_inner()
                    .try_into()?;

                let end_height = self.latest_known_block_height().await?.0;

                let mut stream = client
                    .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
                        chain_id: params.chain_id,
                        start_height: 0,
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

                while let Some(block_rsp) = stream.message().await? {
                    let block: CompactBlock = block_rsp.try_into()?;
                    progress_bar.set_position(block.height);
                }
                progress_bar.finish();
            }
        }

        Ok(())
    }

    // This code is ripped from the view service code, and could be split out into something common.
    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> Result<(u64, bool), anyhow::Error> {
        let client = reqwest::Client::new();

        let rsp: serde_json::Value = client
            .get(format!(
                r#"http://{}:{}/status"#,
                self.node, self.tendermint_port
            ))
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
