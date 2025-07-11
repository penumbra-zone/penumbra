use anyhow::anyhow;
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext, Version},
    AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_asset::Value;
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::{
    core::component::shielded_pool::v1::{
        self as pb, event_outbound_fungible_token_refund::Reason as RefundReason,
    },
    event::ProtoEvent as _,
};

/// The kind of event we might care about.
#[derive(Clone, Copy, Debug)]
enum EventKind {
    InboundTransfer,
    OutboundTransfer,
    OutboundRefund,
}

impl EventKind {
    fn tag(&self) -> &'static str {
        match self {
            Self::InboundTransfer => {
                "penumbra.core.component.shielded_pool.v1.EventInboundFungibleTokenTransfer"
            }
            Self::OutboundTransfer => {
                "penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenTransfer"
            }
            Self::OutboundRefund => {
                "penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenRefund"
            }
        }
    }
}

impl TryFrom<&str> for EventKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for kind in [
            Self::InboundTransfer,
            Self::OutboundTransfer,
            Self::OutboundRefund,
        ] {
            if kind.tag() == value {
                return Ok(kind);
            }
        }
        Err(anyhow!("unexpected event kind: {value}"))
    }
}

/// Represents the event data that we care about.
#[derive(Debug, Clone)]
enum Event {
    InboundTransfer {
        receiver: Address,
        sender: String,
        value: Value,
    },
    OutboundTransfer {
        sender: Address,
        receiver: String,
        value: Value,
    },
    OutboundRefund {
        sender: Address,
        receiver: String,
        value: Value,
        reason: RefundReason,
    },
}

impl TryFrom<ContextualizedEvent<'_>> for Event {
    type Error = anyhow::Error;

    fn try_from(event: ContextualizedEvent<'_>) -> Result<Self, Self::Error> {
        match EventKind::try_from(event.event.kind.as_str())? {
            EventKind::InboundTransfer => {
                let pe = pb::EventInboundFungibleTokenTransfer::from_event(&event.event)?;
                Ok(Self::InboundTransfer {
                    receiver: pe.receiver.ok_or(anyhow!("missing receiver"))?.try_into()?,
                    sender: pe.sender,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                })
            }
            EventKind::OutboundTransfer => {
                let pe = pb::EventOutboundFungibleTokenTransfer::from_event(&event.event)?;
                Ok(Self::OutboundTransfer {
                    sender: pe.sender.ok_or(anyhow!("missing sender"))?.try_into()?,
                    receiver: pe.receiver,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                })
            }
            EventKind::OutboundRefund => {
                let pe = pb::EventOutboundFungibleTokenRefund::from_event(&event.event)?;
                let reason = pe.reason();
                Ok(Self::OutboundRefund {
                    sender: pe.sender.ok_or(anyhow!("missing sender"))?.try_into()?,
                    receiver: pe.receiver,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                    reason,
                })
            }
        }
    }
}

/// The database's view of a transfer.
#[derive(Debug)]
struct DatabaseTransfer {
    penumbra_addr: Address,
    foreign_addr: String,
    negate: bool,
    value: Value,
    kind: &'static str,
}

impl Event {
    fn db_transfer(self) -> DatabaseTransfer {
        match self {
            Event::InboundTransfer {
                receiver,
                sender,
                value,
            } => DatabaseTransfer {
                penumbra_addr: receiver,
                foreign_addr: sender,
                negate: false,
                value,
                kind: "inbound",
            },
            Event::OutboundTransfer {
                sender,
                receiver,
                value,
            } => DatabaseTransfer {
                penumbra_addr: sender,
                foreign_addr: receiver,
                negate: true,
                value,
                kind: "outbound",
            },
            Event::OutboundRefund {
                sender,
                receiver,
                value,
                reason,
            } => DatabaseTransfer {
                penumbra_addr: sender,
                foreign_addr: receiver,
                negate: false,
                value,
                kind: match reason {
                    RefundReason::Unspecified => "refund_other",
                    RefundReason::Timeout => "refund_timeout",
                    RefundReason::Error => "refund_error",
                },
            },
        }
    }
}

async fn init_db(dbtx: &mut PgTransaction<'_>) -> anyhow::Result<()> {
    for statement in include_str!("ibc.sql").split(";") {
        sqlx::query(statement).execute(dbtx.as_mut()).await?;
    }
    Ok(())
}

async fn create_transfer(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    transfer: DatabaseTransfer,
    tx_hash: &[u8],
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO ibc_transfer VALUES (DEFAULT, $7, $1, $6::NUMERIC(39, 0) * $2::NUMERIC(39, 0), $3, $4, $5, $8)")
        .bind(transfer.value.asset_id.to_bytes())
        .bind(transfer.value.amount.to_string())
        .bind(transfer.penumbra_addr.to_vec())
        .bind(transfer.foreign_addr)
        .bind(transfer.kind)
        .bind(if transfer.negate { -1i32 } else { 1i32 })
        .bind(i64::try_from(height)?)
        .bind(tx_hash.to_vec())
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
    fn name(&self) -> String {
        "ibc".to_string()
    }

    fn version(&self) -> Version {
        Version::with_major(1)
    }

    async fn reset(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("reset.sql").split(";") {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(dbtx.as_mut()).await?;
            }
        }
        Ok(())
    }

    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> anyhow::Result<()> {
        init_db(dbtx).await
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            let parsed = match Event::try_from(event) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let transfer = parsed.db_transfer();
            create_transfer(
                dbtx,
                event.block_height,
                transfer,
                &event
                    .tx_hash()
                    .expect("IBC events should have associated transaction hash"),
            )
            .await?;
        }
        Ok(())
    }
}
