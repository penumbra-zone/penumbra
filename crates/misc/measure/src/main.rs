#[macro_use]
extern crate tracing;

use clap::Parser;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

use penumbra_chain::params::ChainParameters;
use penumbra_compact_block::CompactBlock;
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_service_client::ObliviousQueryServiceClient,
        tendermint_proxy_service_client::TendermintProxyServiceClient, ChainParametersRequest,
        CompactBlockRangeRequest, GetStatusRequest,
    },
    Message,
};

use tonic::transport::{Channel, ClientTlsConfig};
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
        default_value = "https://grpc.testnet.penumbra.zone",
        env = "PENUMBRA_NODE_PD_URL",
        parse(try_from_str = url::Url::parse)
    )]
    node: Url,
    #[clap(subcommand)]
    pub cmd: Command,
    /// The filter for log messages.
    #[clap( long, default_value_t = EnvFilter::new("warn,measure=info"), env = "RUST_LOG")]
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
    /// Load-test `pd` by holding open many connections subscribing to compact block updates.
    OpenConnections {
        /// The number of connections to open.
        #[clap(short, long, default_value = "100")]
        num_connections: usize,

        /// Whether to sync the entire chain state, then exit.
        #[clap(long)]
        full_sync: bool,
    },
}

impl Opt {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.cmd {
            Command::OpenConnections {
                num_connections,
                full_sync,
            } => {
                let current_height = self.latest_known_block_height().await?.0;
                // Configure start/stop ranges on query, depending on whether we want a full sync.
                let start_height = if full_sync { 0 } else { current_height };
                let end_height = if full_sync { current_height } else { 0 };
                let node = self.node.to_string();
                let mut js = tokio::task::JoinSet::new();
                for conn_id in 0..num_connections {
                    let node2 = node.clone();
                    js.spawn(
                        async move {
                            let mut client =
                                ObliviousQueryServiceClient::connect(node2).await.unwrap();

                            let mut stream = client
                                .compact_block_range(tonic::Request::new(
                                    CompactBlockRangeRequest {
                                        chain_id: String::new(),
                                        start_height: start_height,
                                        end_height: end_height,
                                        keep_alive: true,
                                    },
                                ))
                                .await
                                .unwrap()
                                .into_inner();
                            while let Some(block_rsp) = stream.message().await.unwrap() {
                                let size = block_rsp.encoded_len();
                                let block: CompactBlock = block_rsp.try_into().unwrap();
                                tracing::debug!(block_size = ?size, block_height = ?block.height, initial_chain_height = ?current_height);
                                // Exit if we only wanted a single full sync per client.
                                if full_sync && block.height >=  current_height {
                                    break;
                                }
                            }
                        }
                        .instrument(debug_span!("open-connection", conn_id = conn_id)),
                    );
                }
                while let Some(res) = js.join_next().await {
                    res?;
                }
            }
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

    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> Result<(u64, bool), anyhow::Error> {
        let mut client = get_tendermint_proxy_client(self.node.clone()).await?;
        let rsp = client.get_status(GetStatusRequest {}).await?.into_inner();
        let sync_info = rsp
            .sync_info
            .ok_or_else(|| anyhow::anyhow!("could not parse sync_info in gRPC response"))?;

        let latest_block_height = sync_info.latest_block_height;
        let node_catching_up = sync_info.catching_up;
        Ok((latest_block_height, node_catching_up))
    }
}

// This code is ripped from the pcli code, and could be split out into something common.
async fn get_tendermint_proxy_client(
    pd_url: Url,
) -> Result<TendermintProxyServiceClient<Channel>, anyhow::Error> {
    let pd_channel: Channel = match pd_url.scheme() {
        "http" => Channel::from_shared(pd_url.to_string())?.connect().await?,
        "https" => {
            Channel::from_shared(pd_url.to_string())?
                .tls_config(ClientTlsConfig::new())?
                .connect()
                .await?
        }
        other => anyhow::bail!(format!("unknown url scheme {other}")),
    };
    Ok(TendermintProxyServiceClient::new(pd_channel))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut opt = Opt::parse();
    opt.init_tracing();
    opt.run().await?;
    Ok(())
}
