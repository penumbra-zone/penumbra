use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {}

#[tokio::main]
async fn main() {
    let _opt = Opt::from_args();
}
