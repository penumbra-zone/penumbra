#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate clap;

use tracing_subscriber::EnvFilter;
use url::Url;

use penumbra_proto::client::oblivious::{
    oblivious_query_client::ObliviousQueryClient, ChainParamsRequest,
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

#[derive(Parser)]
enum Command {
    /// Measure the performance of downloading compact blocks without parsing them.
    StreamBlocks,
}

#[derive(Debug)]
struct App {
    pub pd_url: Url,
    pub tendermint_url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ObliviousQueryClient::connect(format!("http://{}:{}", node, pd_port)).await?;

    let params = client
        .chain_params(tonic::Request::new(ChainParamsRequest {
            chain_id: String::new(),
        }))
        .await?
        .into_inner()
        .try_into()?;

    Ok(())
}
