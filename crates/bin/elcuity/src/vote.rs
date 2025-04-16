use clap::Args;

#[derive(Debug, Args)]
pub struct Opt {
    /// The denom that should continuously be voted for.
    #[clap(long = "for")]
    denom: String,
}

impl Opt {
    pub async fn run(self) -> anyhow::Result<()> {
        todo!()
    }
}
