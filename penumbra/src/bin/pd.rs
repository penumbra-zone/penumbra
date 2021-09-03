use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pd",
    about = "The Penumbra daemon.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {}

#[tokio::main]
async fn main() {
    let _opt = Opt::from_args();
}
