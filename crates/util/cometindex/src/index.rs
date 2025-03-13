use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use async_trait::async_trait;
pub use sqlx::PgPool;
use sqlx::{Postgres, Transaction};
use tendermint::abci::Event;

use crate::ContextualizedEvent;

pub type PgTransaction<'a> = Transaction<'a, Postgres>;

#[derive(Clone, Copy, Debug)]
struct EventReference {
    /// Which event in the block this is.
    pub event_index: usize,
    pub tx_hash: Option<[u8; 32]>,
    pub local_rowid: i64,
}

/// Represents all of the events in a given block
#[derive(Clone, Debug)]
pub struct BlockEvents {
    height: u64,
    event_refs: Vec<EventReference>,
    events: Vec<Event>,
    transactions: BTreeMap<[u8; 32], Vec<u8>>,
}

// The builder interface for our own crate.
impl BlockEvents {
    pub(crate) fn new(height: u64) -> Self {
        const EXPECTED_EVENTS: usize = 32;

        Self {
            height,
            event_refs: Vec::with_capacity(EXPECTED_EVENTS),
            events: Vec::with_capacity(EXPECTED_EVENTS),
            transactions: BTreeMap::new(),
        }
    }

    /// Register a transaction in this block.
    pub(crate) fn push_tx(&mut self, hash: [u8; 32], data: Vec<u8>) {
        self.transactions.insert(hash, data);
    }

    /// Register an event in this block.
    pub(crate) fn push_event(&mut self, event: Event, tx_hash: Option<[u8; 32]>, local_rowid: i64) {
        let event_index = self.events.len();
        self.events.push(event);
        self.event_refs.push(EventReference {
            event_index,
            tx_hash,
            local_rowid,
        });
    }
}

impl BlockEvents {
    pub fn height(&self) -> u64 {
        self.height
    }

    fn contextualize(&self, event_ref: EventReference) -> ContextualizedEvent<'_> {
        let event = &self.events[event_ref.event_index];
        let tx = event_ref
            .tx_hash
            .and_then(|h| Some((h, self.transactions.get(&h)?.as_slice())));
        ContextualizedEvent {
            event,
            block_height: self.height,
            tx,
            local_rowid: event_ref.local_rowid,
        }
    }

    /// Iterate over the events in this block, in the order that they appear.
    pub fn events(&self) -> impl Iterator<Item = ContextualizedEvent<'_>> {
        self.event_refs.iter().map(|x| self.contextualize(*x))
    }

    /// Iterate over transactions (and their hashes) in the order they appear in the block.
    pub fn transactions(&self) -> impl Iterator<Item = ([u8; 32], &'_ [u8])> {
        self.transactions.iter().map(|x| (*x.0, x.1.as_slice()))
    }
}

#[derive(Clone, Debug)]
pub struct EventBatch {
    first_height: u64,
    last_height: u64,
    /// The batch of events, ordered by increasing height.
    ///
    /// The heights are guaranteed to be increasing, and to be contiguous.
    by_height: Arc<Vec<BlockEvents>>,
}

impl EventBatch {
    /// Create a new [`EventBatch`].
    pub fn new(block_events: Vec<BlockEvents>) -> Self {
        Self {
            first_height: block_events.first().map(|x| x.height).unwrap_or_default(),
            last_height: block_events.last().map(|x| x.height).unwrap_or_default(),
            by_height: Arc::new(block_events),
        }
    }

    pub(crate) fn first_height(&self) -> u64 {
        self.first_height
    }

    pub(crate) fn last_height(&self) -> u64 {
        self.last_height
    }

    /// Check if this batch has no blocks in it.
    ///
    /// Most commonly, this is the result when [`start_later`] is called with a height
    /// past that inside the batch.
    pub fn empty(&self) -> bool {
        self.first_height > self.last_height
    }

    /// Modify this batch to start at a greater height.
    ///
    /// This will have no effect if the new start height is *before* the current start height.
    pub fn start_later(&mut self, new_start: u64) {
        self.first_height = new_start.max(self.first_height);
    }

    pub fn events_by_block(&self) -> impl Iterator<Item = &'_ BlockEvents> {
        // Assuming the first height is past the first height in our vec,
        // we need to skip the difference.
        let skip = self
            .by_height
            .first()
            .map(|x| self.first_height.saturating_sub(x.height) as usize)
            .unwrap_or_default();
        self.by_height.iter().skip(skip)
    }

    pub fn events(&self) -> impl Iterator<Item = ContextualizedEvent<'_>> {
        self.events_by_block().flat_map(|x| x.events())
    }
}

/// Provides more information about the context a batch of event comes from.
#[derive(Debug, Clone)]
pub struct EventBatchContext {
    pub(crate) is_last: bool,
}

impl EventBatchContext {
    /// If true, then no further event batches will be sent before new blocks arrive.
    pub fn is_last(&self) -> bool {
        self.is_last
    }
}

/// Represents the version of an indexing view.
///
/// This allows tracking breaking change at the level of each individual view,
/// rather than on the level of the database as a whole.
///
/// Versions can be compared to assess breaking changes:
/// ```
/// assert!(Version::with_major(3) > Version::with_major(2));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
#[sqlx(transparent)]
pub struct Version(Option<i64>);

impl Version {
    /// Construct a new version by specifying a "major" / "breaking" number.
    pub fn with_major(v: u64) -> Self {
        Self(Some(v.try_into().expect("version must fit into an i64")))
    }

    /// Get the major version, which controls breakage.
    pub fn major(self) -> u64 {
        u64::try_from(self.0.unwrap_or_default()).expect("major version cannot be negative")
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0.unwrap_or_default())
    }
}

/// Represents a specific index of raw event data.
#[async_trait]
pub trait AppView: Send + Sync {
    /// Return the name of this index.
    ///
    /// This should be unique across all of the indices.
    fn name(&self) -> String;

    /// Return the version of this index.
    ///
    /// As code evolves, versions should increase, only.
    /// If one version is greater than another, that indicates that the view
    /// needs to be reindexed.
    fn version(&self) -> Version {
        Version::default()
    }

    /// Reset this app view to an empty state.
    ///
    /// This should delete all tables, across all versions, resetting the
    /// app view to a blank state.
    async fn reset(&self, _dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        unimplemented!(
            r#"
Index {} has not implemented `reset` despite being on version {}.
For versions > v0, this method needs to be implemented, so that we know
how to delete previous versions of the schema.
"#,
            self.name(),
            self.version()
        )
    }

    async fn on_startup(&self, _dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// This will be called once when processing the genesis before the first block.
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error>;

    /// This allows processing a batch of events, over many blocks.
    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        context: EventBatchContext,
    ) -> Result<(), anyhow::Error>;
}
