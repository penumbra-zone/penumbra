//! Logic for determining protocol compatibility, as defined by the
//! workspace-wide APP_VERSION value. On startup, `pd` will inspect
//! the APP_VERSION set in the local non-verifiable storage, and warn
//! or fail about mismatches.
//!
//!
//! TODO: move this code into the `app` crate.
use anyhow;
use cnidarium::{StateDelta, StateRead, StateWrite, Storage};
// use penumbra_app::app::StateReadExt as _;
use penumbra_app::APP_VERSION;

const APP_VERSION_KEY: &str = "app_version";

/// Retrieve the APP_VERSION last written to local non-verifiable storage.
/// Returns an Option, because as late as v0.80.8, APP_VERSION was not
/// written to local state.
pub async fn get_state_version(storage: Storage) -> anyhow::Result<Option<u64>> {
    let snapshot = storage.latest_snapshot();
    let result = snapshot
        .nonverifiable_get_raw(APP_VERSION_KEY.as_bytes())
        .await?;

    let local_app_version = match result {
        Some(v) => {
            let app_version: AppVersion = v.try_into()?;
            Some(app_version.try_into()?)
        }
        None => None,
    };
    Ok(local_app_version)
}

/// Check whether the local state matches the current APP_VERSION.
/// If it's not set yet, set it to the current value.
/// If it's a surprising value, error.
pub async fn check_state_version(storage: Storage) -> anyhow::Result<u64> {
    let local_version = get_state_version(storage.clone()).await?;
    match local_version {
        Some(v) => {
            if v > APP_VERSION {
                anyhow::bail!(
                    "state version {v} is newer than current app version {APP_VERSION}; you should upgrade pd",
                )
            } else if v < APP_VERSION {
                anyhow::bail!(
                    "state version {v} is older than current app version {APP_VERSION}; you should run 'pd migrate'",
                )
            } else {
                Ok(v)
            }
        }
        // If not set, set it.
        None => {
            set_state_version(storage, APP_VERSION).await?;
            Ok(APP_VERSION)
        }
    }
}

/// Write the given version to the local non-verifiable storage.
pub async fn set_state_version(storage: Storage, version: u64) -> anyhow::Result<()> {
    let v = AppVersion(version);
    let snapshot = storage.latest_snapshot();
    let mut delta = StateDelta::new(snapshot);
    delta.nonverifiable_put_raw(APP_VERSION_KEY.to_string().into(), v.try_into()?);
    Ok(())
}

/// Wrapper struct representing the `APP_VERSION`, intended
/// for custom TryInto/TryFrom implementations.
#[derive(Debug)]
struct AppVersion(u64);

use std::convert::TryFrom;

impl From<u64> for AppVersion {
    fn from(i: u64) -> Self {
        Self(i)
    }
}

impl From<AppVersion> for u64 {
    fn from(v: AppVersion) -> Self {
        v.0
    }
}

/// Ensure that bytes from be read out of non-verifiable [Storage]
/// and interpreted as an AppVersion.
impl TryFrom<Vec<u8>> for AppVersion {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> anyhow::Result<Self> {
        if bytes.len() > 8 {
            anyhow::bail!("jawn is bad");
        }

        let mut result: u64 = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            result |= (byte as u64) << (i * 8);
        }
        Ok(Self(result))
    }
}

/// Ensure that an AppVersion can be written to non-verifiable [Storage] as bytes.
impl TryFrom<AppVersion> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(value: AppVersion) -> anyhow::Result<Self> {
        let mut bytes = Vec::with_capacity(8);
        let mut remaining = value.0;

        while remaining > 0 || bytes.is_empty() {
            bytes.push((remaining & 0xFF) as u8);
            remaining >>= 8;
        }

        Ok(bytes)
    }
}
