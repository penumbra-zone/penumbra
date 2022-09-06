//! A web service to view the live state of the TCT.

use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    headers::ContentType,
    http::StatusCode,
    routing::{get, post, MethodRouter},
    Json, Router, TypedHeader,
};

use parking_lot::Mutex;
use rand::{seq::SliceRandom, Rng};
use serde_json::json;
use tokio::sync::watch;

use crate::{
    builder::{block, epoch},
    Commitment, Forgotten, Position, Tree, Witness,
};

mod earliest;
use earliest::Earliest;

mod view;
pub use view::view;

mod query;
pub use query::query;

mod control;
pub use control::control;
