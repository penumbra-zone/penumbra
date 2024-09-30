use anyhow::anyhow;
use cometindex::{async_trait, AppView, ContextualizedEvent, PgTransaction};
use penumbra_auction::auction::{
    dutch::{DutchAuctionDescription, DutchAuctionState},
    AuctionId,
};
use penumbra_proto::{core::component::auction::v1 as pb, event::ProtoEvent};
use sqlx::PgPool;

#[derive(Clone, Copy)]
enum EventKind {
    DutchScheduled,
    DutchUpdated,
    DutchEnded,
    DutchWithdrawn,
}

impl EventKind {
    /// The string tag of this kind of event.
    ///
    /// This is used to match the raw ABCI events with this kind.
    fn tag(&self) -> &'static str {
        const TAGS: [&'static str; 4] = [
            "penumbra.core.component.auction.v1.EventDutchAuctionScheduled",
            "penumbra.core.component.auction.v1.EventDutchAuctionUpdated",
            "penumbra.core.component.auction.v1.EventDutchAuctionEnded",
            "penumbra.core.component.auction.v1.EventDutchAuctionWithdrawn",
        ];
        let ix: usize = match *self {
            EventKind::DutchScheduled => 0,
            EventKind::DutchUpdated => 1,
            EventKind::DutchEnded => 2,
            EventKind::DutchWithdrawn => 3,
        };
        TAGS[ix]
    }
}

impl TryFrom<&str> for EventKind {
    type Error = anyhow::Error;

    fn try_from(tag: &str) -> Result<Self, Self::Error> {
        use EventKind::*;

        for kind in [DutchScheduled, DutchUpdated, DutchEnded, DutchWithdrawn] {
            if tag == kind.tag() {
                return Ok(kind);
            }
        }
        return Err(anyhow!("unexpected event kind: {tag}"));
    }
}

#[derive(Debug, Clone)]
enum Event {
    DutchScheduled {
        id: AuctionId,
        description: DutchAuctionDescription,
    },
    DutchUpdated {
        id: AuctionId,
        state: DutchAuctionState,
    },
    DutchEnded {
        id: AuctionId,
        state: DutchAuctionState,
        // We don't have anything better to represent this with.
        reason: pb::event_dutch_auction_ended::Reason,
    },
    DutchWithdrawn {
        id: AuctionId,
        state: DutchAuctionState,
    },
}

impl TryFrom<&ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &ContextualizedEvent) -> Result<Self, Self::Error> {
        match EventKind::try_from(event.event.kind.as_str())? {
            EventKind::DutchScheduled => {
                let pe = pb::EventDutchAuctionScheduled::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let description = pe
                    .description
                    .ok_or(anyhow!("missing description"))?
                    .try_into()?;
                Ok(Event::DutchScheduled { id, description })
            }
            EventKind::DutchUpdated => {
                let pe = pb::EventDutchAuctionUpdated::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                Ok(Event::DutchUpdated { id, state })
            }
            EventKind::DutchEnded => {
                let pe = pb::EventDutchAuctionEnded::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                let reason = pe.reason.try_into()?;
                Ok(Event::DutchEnded { id, state, reason })
            }
            EventKind::DutchWithdrawn => {
                let pe = pb::EventDutchAuctionWithdrawn::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                Ok(Event::DutchWithdrawn { id, state })
            }
        }
    }
}

async fn create_dutch_auction_description(
    dbtx: &mut PgTransaction<'_>,
    id: AuctionId,
    description: &DutchAuctionDescription,
) -> anyhow::Result<()> {
    sqlx::query(
        "
    INSERT INTO
        auction_dutch_description 
    VALUES (
        $1,
        $2,
        $3::NUMERIC(39, 0),
        $4,
        $5::NUMERIC(39, 0),
        $6::NUMERIC(39, 0),
        $7,
        $8, 
        $9, 
        $10
    )",
    )
    .bind(id.0)
    .bind(&description.input.asset_id.to_bytes())
    .bind(&description.input.amount.to_string())
    .bind(&description.output_id.to_bytes())
    .bind(&description.max_output.to_string())
    .bind(&description.min_output.to_string())
    .bind(i64::try_from(description.start_height)?)
    .bind(i64::try_from(description.end_height)?)
    .bind(i64::try_from(description.step_count)?)
    .bind(&description.nonce)
    .execute(dbtx.as_mut())
    .await?;
    Ok(())
}

async fn create_dutch_auction_update(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    id: AuctionId,
    state: &DutchAuctionState,
    reason: Option<pb::event_dutch_auction_ended::Reason>,
) -> anyhow::Result<()> {
    sqlx::query(
        "
    INSERT INTO
        auction_dutch_update 
    VALUES (
        DEFAULT,
        $1,
        $2,
        $3,
        $4,
        $5,
        $6,
        $7::NUMERIC(39, 0),
        $8::NUMERIC(39, 0)
    )",
    )
    .bind(&id.0)
    .bind(i64::try_from(height)?)
    .bind(i32::try_from(state.sequence)?)
    .bind(reason.map(|x| {
        use pb::event_dutch_auction_ended::Reason::*;
        match x {
            Unspecified => 0i32,
            Expired => 1,
            Filled => 2,
            ClosedByOwner => 3,
        }
    }))
    .bind(state.current_position.map(|x| x.0))
    .bind(
        state
            .next_trigger
            .map(|x| i64::try_from(u64::from(x)))
            .transpose()?,
    )
    .bind(&state.input_reserves.to_string())
    .bind(&state.output_reserves.to_string())
    .execute(dbtx.as_mut())
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct Component {}

impl Component {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> anyhow::Result<()> {
        for statement in include_str!("auction.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        EventKind::try_from(type_str).is_ok()
    }

    #[tracing::instrument(skip_all, fields(height = event.block_height, name = event.event.kind.as_str()))]
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> anyhow::Result<()> {
        let height = event.block_height;
        match Event::try_from(event)? {
            Event::DutchScheduled { id, description } => {
                create_dutch_auction_description(dbtx, id, &description).await?;
            }
            Event::DutchUpdated { id, state } => {
                create_dutch_auction_update(dbtx, height, id, &state, None).await?;
            }
            Event::DutchEnded { id, state, reason } => {
                create_dutch_auction_update(dbtx, height, id, &state, Some(reason)).await?;
            }
            Event::DutchWithdrawn { id, state } => {
                create_dutch_auction_update(dbtx, height, id, &state, None).await?;
            }
        };
        Ok(())
    }
}
