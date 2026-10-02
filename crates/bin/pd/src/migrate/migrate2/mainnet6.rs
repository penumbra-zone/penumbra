//! Mainnet-6 migration: upgrade to `APP_VERSION` 12 and revive the expired IBC clients
//! behind the original Cosmos Hub, Celestia and Osmosis channels.
//!
//! The clients behind `channel-0`, `channel-3` and `channel-4` have expired, and an expired
//! client can't be updated again, so every asset that left through
//! those channels is stuck until the client is replaced. Relayers have since created fresh
//! clients against the same chains. This migration copies each fresh client's state over
//! its expired predecessor, so the original connections and channels, and the IBC denoms
//! derived from them, work again.
//!
//! The recoveries are hardcoded, so validators run `pd migrate mainnet6` with no
//! arguments and every node applies the same change.
//!
//! Each old client takes on its substitute's parameters, and those differ from the
//! original ones: a 1/3 trust level, a 20–25 s clock drift (down from 7235 s), and
//! Celestia's current 14 day + 1 hour unbonding period. The strict ADR-26 check would
//! refuse that, so these recoveries use
//! [`RecoveryPolicy::AdoptSubstituteParameters`], which pins the counterparty chain ID
//! instead. All the other safety checks still apply: the old client must not be Active,
//! the substitute must be Active at the halt height, and it must be ahead of the old one.
//!
//! If any of the three is refused, the whole migration fails, and no node produces a
//! genesis with only some of the recoveries applied.

use anyhow::{Context, Result};
use cnidarium::StateDelta;
use ibc_types::core::client::ClientId;
use penumbra_sdk_app::{PenumbraHost, APP_VERSION};
use penumbra_sdk_ibc::component::{ClientRecoveryExt, RecoveryPolicy};

use super::framework::Migration;

/// One hardcoded client recovery.
#[derive(Debug, Clone, Copy)]
pub struct Recovery {
    /// The counterparty chain ID both clients must track.
    pub chain_id: &'static str,
    /// The expired client that the original channel's connection is built on.
    pub subject: &'static str,
    /// The live client whose state replaces it.
    pub substitute: &'static str,
}

/// The recoveries this upgrade applies, checked against mainnet state on 2026-09-25.
pub const RECOVERIES: [Recovery; 3] = [
    // channel-0. Trust level 2/3 -> 1/3, clock drift 7235s -> 25s.
    Recovery {
        chain_id: "cosmoshub-4",
        subject: "07-tendermint-0",
        substitute: "07-tendermint-29",
    },
    // channel-3. Unbonding period 21d -> 14d 1h (Celestia CIP-037), clock drift 7235s -> 25s.
    Recovery {
        chain_id: "celestia",
        subject: "07-tendermint-3",
        substitute: "07-tendermint-30",
    },
    // channel-4. Clock drift 7235s -> 20s.
    Recovery {
        chain_id: "osmosis-1",
        subject: "07-tendermint-4",
        substitute: "07-tendermint-27",
    },
];

pub struct Mainnet6Migration;

impl Migration for Mainnet6Migration {
    fn name(&self) -> &'static str {
        "mainnet-6"
    }

    fn target_app_version(&self) -> Option<u64> {
        Some(APP_VERSION)
    }

    async fn migrate_inner(&self, delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        for recovery in RECOVERIES {
            let subject: ClientId = recovery.subject.parse()?;
            let substitute: ClientId = recovery.substitute.parse()?;
            tracing::info!(?recovery, "recovering ibc client");
            delta
                .recover_client_with_policy::<PenumbraHost>(
                    &subject,
                    &substitute,
                    &RecoveryPolicy::AdoptSubstituteParameters {
                        expected_chain_id: recovery.chain_id.to_string(),
                    },
                )
                .await
                .with_context(|| format!("failed to recover {recovery:?}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn recoveries_are_well_formed() {
        let mut clients = HashSet::new();
        for recovery in RECOVERIES {
            for id in [recovery.subject, recovery.substitute] {
                id.parse::<ClientId>().unwrap();
                assert!(
                    id.starts_with("07-tendermint-"),
                    "{id} is not a tendermint client"
                );
                // A client used twice means two recoveries would overwrite each other.
                assert!(clients.insert(id), "{id} appears in more than one recovery");
            }
        }
    }

    #[test]
    fn targets_the_version_after_mainnet() {
        // Mainnet runs app version 11 (pd 2.0.x). `migrate_app_version` refuses any
        // target other than found + 1, so this pins which chain the binary upgrades.
        assert_eq!(Mainnet6Migration.target_app_version(), Some(12));
    }
}
