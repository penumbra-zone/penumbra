//! Data modeling for Penumbra transactions.
//!
//! This crate defines data structures that provide modeling of shielded
//! transactions through their entire lifecycle:
//!
//! * the [`TransactionPlan`](plan::TransactionPlan) type completely describes a
//! planned transaction before it is created;
//!
//! * the [`Transaction`] type represents the shielded transaction itself;
//!
//! * the [`TransactionView`] type represents a view from a particular
//! [`TransactionPerspective`] (e.g., the sender or receiver) of the cleartext
//! contents of a shielded transaction after it has been created.

#![allow(clippy::clone_on_copy)]

mod auth_data;
mod auth_hash;
mod effect_hash;
mod error;
mod id;
mod is_action;
mod transaction;
mod witness_data;

pub mod action;
pub mod plan;
pub mod proposal;
pub mod view;
pub mod vote;

pub use action::Action;
pub use auth_data::AuthorizationData;
pub use auth_hash::{AuthHash, AuthorizingData};
pub use effect_hash::EffectingData;
pub use error::Error;
pub use id::Id;
pub use is_action::IsAction;
pub use transaction::{Transaction, TransactionBody};
pub use view::{ActionView, MemoView, TransactionPerspective, TransactionView};
pub use witness_data::WitnessData;

/// A compatibility wrapper for trait implementations that are temporarily duplicated
/// in multiple crates as an orphan rule work around until we finish splitting crates (#2288).
pub struct Compat<'a, T>(pub &'a T);
