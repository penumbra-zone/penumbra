use clap::Args;
use penumbra_sdk_view::ViewClient;

#[derive(Debug, Args)]
pub struct Opt {
    /// The denom that should continuously be voted for.
    #[clap(long = "for")]
    denom: String,
}

impl Opt {
    pub async fn run(self, _view: &mut dyn ViewClient) -> anyhow::Result<()> {
        todo!()
    }
}
