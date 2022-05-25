// Required because of NCT type size
#![recursion_limit = "256"]

mod client;
mod metrics;
mod note_record;
mod service;
mod storage;
mod sync;
mod worker;

use worker::Worker;

pub use crate::metrics::register_metrics;
pub use client::ViewClient;
pub use note_record::NoteRecord;
pub use service::ViewService;
pub use storage::Storage;
