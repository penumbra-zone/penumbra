pub use cometindex::{opt::Options, AppView, ContextualizedEvent, Indexer, PgPool, PgTransaction};

mod indexer_ext;
pub use indexer_ext::IndexerExt;
pub mod block;
pub mod dex;
pub mod governance;
pub mod shielded_pool;
mod sql;
pub mod stake;

pub(crate) const PD_COMPAT: &'static str = "Check that your pd and pindexer versions match. See pd compatibility section in README for more information.";
