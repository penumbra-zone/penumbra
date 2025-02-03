use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_dex::component::PositionManager as _;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::component::clock::EpochRead as _;
use penumbra_sdk_sct::CommitmentSource;
use penumbra_sdk_shielded_pool::component::NoteManager as _;
use penumbra_sdk_txhash::TransactionId;

#[async_trait]
/// The bank strictly controls issuance of rewards in the liquidity tournament.
///
/// This ensures that rewards do not exceed the issuance budget, and are immediately
/// debited from the appropriate source (which happens to be the community pool),
/// and credited towards the appropriate destination (i.e. positions or new notes).
pub trait Bank: StateWrite + Sized {
    /// Move a fraction of our issuance budget towards an address, by minting a note.
    async fn reward_to_voter(
        &mut self,
        reward: Amount,
        voter: &Address,
        tx_hash: TransactionId,
    ) -> anyhow::Result<()> {
        if reward == Amount::default() {
            return Ok(());
        }
        let epoch = self
            .get_current_epoch()
            .await
            .expect("should be able to read current epoch");
        self.mint_note(
            Value {
                asset_id: *STAKING_TOKEN_ASSET_ID,
                amount: reward,
            },
            voter,
            CommitmentSource::LiquidityTournamentReward {
                epoch: epoch.index,
                tx_hash,
            },
        )
        .await?;
        Ok(())
    }

    /// Move a fraction of our issuance budget towards a position, increasing its reserves.
    async fn reward_to_position(&mut self, reward: Amount, lp: position::Id) -> anyhow::Result<()> {
        if reward == Amount::default() {
            return Ok(());
        }
        self.reward_position(lp, reward).await?;
        Ok(())
    }
}

impl<T: StateWrite + Sized> Bank for T {}
