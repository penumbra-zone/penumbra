use crate::App;

/// Vote in the current round of the liquidity tournament.
///
/// This will plan a transaction which directs all available voting power to a single asset.
#[derive(Debug, clap::Parser)]
pub struct LqtVoteCmd {
    /// The denom string for the asset being voted for.
    vote: String,
}

impl LqtVoteCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(&self, _app: &mut App) -> anyhow::Result<()> {
        unimplemented!()
    }
}
