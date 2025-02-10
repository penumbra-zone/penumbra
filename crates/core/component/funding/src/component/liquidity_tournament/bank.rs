use anyhow::bail;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::component::PositionManager as _;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_proto::StateWriteProto;
use penumbra_sdk_sct::component::clock::EpochRead as _;
use penumbra_sdk_sct::CommitmentSource;
use penumbra_sdk_shielded_pool::component::NoteManager as _;
use penumbra_sdk_stake::component::ValidatorPoolDeposit;
use penumbra_sdk_stake::IdentityKey;

use penumbra_sdk_txhash::TransactionId;

use crate::event;

#[async_trait]
/// The bank strictly controls issuance of rewards in the liquidity tournament.
///
/// This ensures that rewards do not exceed the issuance budget, and are immediately
/// debited from the appropriate source (which happens to be the community pool),
/// and credited towards the appropriate destination (i.e. positions or new notes).
pub trait Bank: StateWrite + Sized {
    /// Move a fraction of our issuance budget towards an address, by minting a note.
    #[tracing::instrument(skip(self))]
    async fn reward_to_voter(
        &mut self,
        unbonded_reward: Amount,
        validator: IdentityKey,
        voter: &Address,
        tx_hash: TransactionId,
        incentivized_asset_id: asset::Id,
    ) -> anyhow::Result<()> {
        tracing::debug!("rewarding voter");
        if unbonded_reward == Amount::zero() {
            return Ok(());
        }
        let epoch = self
            .get_current_epoch()
            .await
            .expect("should be able to read current epoch");

        let Some(bonded_reward) = self
            .deposit_to_validator_pool(&validator, unbonded_reward)
            .await
        else {
            bail!("failed to deposit to validator pool");
        };

        self.record_proto(
            event::EventLqtDelegatorReward {
                epoch_index: epoch.index,
                delegation_tokens: bonded_reward,
                reward_amount: unbonded_reward,
                address: voter.clone(),
                incentivized_asset_id,
            }
            .to_proto(),
        );

        self.mint_note(
            bonded_reward,
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
    #[tracing::instrument(skip(self))]
    async fn reward_to_position(&mut self, reward: Amount, lp: position::Id) -> anyhow::Result<()> {
        tracing::debug!("rewarding position");
        if reward == Amount::default() {
            return Ok(());
        }
        self.reward_position(lp, reward).await?;
        Ok(())
    }
}

impl<T: StateWrite + Sized> Bank for T {}
