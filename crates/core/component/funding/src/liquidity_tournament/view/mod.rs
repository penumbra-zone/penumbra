use anyhow::{anyhow, Context};
use penumbra_sdk_proto::{core::component::funding::v1 as pb, DomainType};
use penumbra_sdk_shielded_pool::NoteView;
use serde::{Deserialize, Serialize};

use super::ActionLiquidityTournamentVote;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionLiquidityTournamentVoteView",
    into = "pb::ActionLiquidityTournamentVoteView"
)]
#[allow(clippy::large_enum_variant)]
pub enum ActionLiquidityTournamentVoteView {
    Visible {
        vote: ActionLiquidityTournamentVote,
        note: NoteView,
    },
    Opaque {
        vote: ActionLiquidityTournamentVote,
    },
}

impl DomainType for ActionLiquidityTournamentVoteView {
    type Proto = pb::ActionLiquidityTournamentVoteView;
}

impl TryFrom<pb::ActionLiquidityTournamentVoteView> for ActionLiquidityTournamentVoteView {
    type Error = anyhow::Error;

    fn try_from(value: pb::ActionLiquidityTournamentVoteView) -> Result<Self, Self::Error> {
        let out: Result<Self, Self::Error> = match value
            .liquidity_tournament_vote
            .ok_or_else(|| anyhow::anyhow!("missing `liquidity_tournament_vote`"))?
        {
            pb::action_liquidity_tournament_vote_view::LiquidityTournamentVote::Visible(
                visible,
            ) => Ok(Self::Visible {
                vote: visible
                    .vote
                    .ok_or_else(|| anyhow!("missing `visible.vote`"))?
                    .try_into()?,
                note: visible
                    .note
                    .ok_or_else(|| anyhow!("missing `visible.note`"))?
                    .try_into()?,
            }),
            pb::action_liquidity_tournament_vote_view::LiquidityTournamentVote::Opaque(opaque) => {
                Ok(Self::Opaque {
                    vote: opaque
                        .vote
                        .ok_or_else(|| anyhow!("missing `opaque.vote`"))?
                        .try_into()?,
                })
            }
        };
        out.with_context(|| format!("while parsing `{}`", std::any::type_name::<Self>()))
    }
}

impl From<ActionLiquidityTournamentVoteView> for pb::ActionLiquidityTournamentVoteView {
    fn from(value: ActionLiquidityTournamentVoteView) -> Self {
        use pb::action_liquidity_tournament_vote_view as pblqtvv;
        match value {
            ActionLiquidityTournamentVoteView::Visible { vote, note } => Self {
                liquidity_tournament_vote: Some(pblqtvv::LiquidityTournamentVote::Visible(
                    pblqtvv::Visible {
                        vote: Some(vote.into()),
                        note: Some(note.into()),
                    },
                )),
            },
            ActionLiquidityTournamentVoteView::Opaque { vote } => Self {
                liquidity_tournament_vote: Some(pblqtvv::LiquidityTournamentVote::Opaque(
                    pblqtvv::Opaque {
                        vote: Some(vote.into()),
                    },
                )),
            },
        }
    }
}

impl From<ActionLiquidityTournamentVoteView> for ActionLiquidityTournamentVote {
    fn from(value: ActionLiquidityTournamentVoteView) -> Self {
        match value {
            ActionLiquidityTournamentVoteView::Visible { vote, .. } => vote,
            ActionLiquidityTournamentVoteView::Opaque { vote } => vote,
        }
    }
}
