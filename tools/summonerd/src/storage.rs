use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use camino::Utf8Path;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proof_setup::{
    all::{
        AllExtraTransitionInformation, Phase1CeremonyCRS, Phase1CeremonyContribution,
        Phase1RawCeremonyCRS, Phase1RawCeremonyContribution, Phase2CeremonyCRS,
        Phase2CeremonyContribution, Phase2RawCeremonyCRS, Phase2RawCeremonyContribution,
    },
    single::log::Hashable,
};
use penumbra_sdk_proto::{
    penumbra::tools::summoning::v1::{
        self as pb, participate_request::Contribution as PBContribution,
    },
    Message,
};
use r2d2_sqlite::{
    rusqlite::{OpenFlags, OptionalExtension},
    SqliteConnectionManager,
};
use tokio::task::spawn_blocking;

use crate::{config::Config, penumbra_knower::PenumbraKnower, phase::PhaseMarker};

/// The current time as a unix timestamp.
///
/// This is used 3 times in this file, so worth abstracting.
///
/// This will return 0 if---for whatever reason---this code is being run in an environment
/// that thinks it's before 1970.
fn current_time_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|x| x.as_secs())
        .unwrap_or(0)
}

/// Represents the possible outcomes of checking contribution eligibility.
#[derive(Clone, Debug)]
pub enum ContributionAllowed {
    Yes(Amount),
    DidntBidEnough(Amount),
    AlreadyContributed,
    Banned,
}

#[derive(Clone)]
pub struct Storage {
    config: Config,
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(
        config: Config,
        storage_path: impl AsRef<Utf8Path>,
    ) -> anyhow::Result<Self> {
        if storage_path.as_ref().exists() {
            return Self::load(config, storage_path).await;
        }

        Self::initialize(config, storage_path).await
    }

    /// Initialize creates the database, but does not insert anything into it.
    async fn initialize(
        config: Config,
        storage_path: impl AsRef<Utf8Path>,
    ) -> anyhow::Result<Self> {
        // Connect to the database (or create it)
        let pool = Self::connect(storage_path)?;

        spawn_blocking(move || {
            // In one database transaction, populate everything
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Create the tables
            tx.execute_batch(include_str!("storage/schema-new.sql"))?;

            tx.commit()?;

            Ok(Storage { config, pool })
        })
        .await?
    }

    async fn load(config: Config, path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        let storage = Self {
            config,
            pool: Self::connect(path)?,
        };

        Ok(storage)
    }

    /// Set the root we need for phase1.
    pub async fn set_root(&mut self, phase_1_root: Phase1CeremonyCRS) -> anyhow::Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO phase1_contribution_data VALUES (0, ?1)",
            (pb::CeremonyCrs::try_from(phase_1_root)?.encode_to_vec(),),
        )?;
        tx.execute(
            "INSERT INTO phase1_contributions VALUES (0, 1, NULL, NULL, ?1)",
            (current_time_unix(),),
        )?;

        tx.commit()?;

        Ok(())
    }

    /// Set the transition information we need.
    pub async fn set_transition(
        &mut self,
        phase_2_root: Phase2CeremonyCRS,
        extra_information: AllExtraTransitionInformation,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO phase2_contribution_data VALUES (0, ?1)",
            (pb::CeremonyCrs::try_from(phase_2_root)?.encode_to_vec(),),
        )?;
        tx.execute(
            "INSERT INTO phase2_contributions VALUES (0, 1, NULL, NULL, ?1)",
            (current_time_unix(),),
        )?;
        tx.execute(
            "INSERT INTO transition_aux VALUES (0, ?1)",
            [extra_information.to_bytes()?],
        )?;

        tx.commit()?;

        Ok(())
    }

    fn connect(path: impl AsRef<Utf8Path>) -> anyhow::Result<r2d2::Pool<SqliteConnectionManager>> {
        let manager = SqliteConnectionManager::file(path.as_ref())
            .with_flags(
                // Don't allow opening URIs, because they can change the behavior of the database; we
                // just want to open normal filepaths.
                OpenFlags::default() & !OpenFlags::SQLITE_OPEN_URI,
            )
            .with_init(|conn| {
                // We use `prepare_cached` a fair amount: this is an overestimate of the number
                // of cached prepared statements likely to be used.
                conn.set_prepared_statement_cache_capacity(32);
                Ok(())
            });
        Ok(r2d2::Pool::new(manager)?)
    }

    pub async fn strike(&self, address: &Address) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        tx.execute(
        "INSERT INTO participant_metadata VALUES(?1, 1) ON CONFLICT(address) DO UPDATE SET strikes = strikes + 1;",
            [
            address.to_vec()
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    async fn get_strikes(&self, address: &Address) -> Result<u64> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let out = tx
            .query_row(
                "SELECT strikes FROM participant_metadata WHERE address = ?1",
                [address.to_vec()],
                |row| row.get::<usize, u64>(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(out)
    }

    /// Check if a participant can contribute.
    ///
    /// If they can't, None will be returned, otherwise we'll have Some(amount),
    /// with the amount indicating their bid, which can be useful for ranking.
    pub async fn can_contribute(
        &self,
        knower: &PenumbraKnower,
        address: &Address,
        marker: PhaseMarker,
    ) -> Result<ContributionAllowed> {
        // Criteria:
        // - Bid more than min amount
        // - Hasn't already contributed
        // - Not banned
        let amount = knower.total_amount_sent_to_me(address).await?;
        if amount < Amount::from(self.config.min_bid_u64) {
            return Ok(ContributionAllowed::DidntBidEnough(amount));
        }
        let has_contributed = {
            let mut conn = self.pool.get()?;
            let tx = conn.transaction()?;
            let query = match marker {
                PhaseMarker::P1 => "SELECT 1 FROM phase1_contributions WHERE address = ?1",
                PhaseMarker::P2 => "SELECT 1 FROM phase2_contributions WHERE address = ?1",
            };
            tx.query_row(query, [address.to_vec()], |_| Ok(()))
                .optional()?
                .is_some()
        };
        if has_contributed {
            return Ok(ContributionAllowed::AlreadyContributed);
        }
        if self.get_strikes(address).await? >= self.config.max_strikes {
            return Ok(ContributionAllowed::Banned);
        }
        Ok(ContributionAllowed::Yes(amount))
    }

    pub async fn phase1_current_crs(&self) -> Result<Option<Phase1CeremonyCRS>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let maybe_data = tx
            .query_row(
                "SELECT p1.is_root, p1_data.contribution_or_crs
            FROM phase1_contributions AS p1
            JOIN phase1_contribution_data AS p1_data ON p1.slot = p1_data.slot
            ORDER BY p1.slot DESC LIMIT 1",
                [],
                |row| Ok((row.get::<usize, bool>(0)?, row.get::<usize, Vec<u8>>(1)?)),
            )
            .optional()?;
        let (is_root, contribution_or_crs) = match maybe_data {
            None => return Ok(None),
            Some(x) => x,
        };
        let crs = if is_root {
            Phase1RawCeremonyCRS::unchecked_from_protobuf(pb::CeremonyCrs::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
        } else {
            Phase1RawCeremonyContribution::unchecked_from_protobuf(PBContribution::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
            .new_elements()
        };
        Ok(Some(crs))
    }

    pub async fn phase2_current_crs(&self) -> Result<Option<Phase2CeremonyCRS>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let maybe_data = tx
            .query_row(
                "SELECT p2.is_root, p2_data.contribution_or_crs
            FROM phase2_contributions AS p2
            JOIN phase2_contribution_data AS p2_data ON p2.slot = p2_data.slot
            ORDER BY p2.slot DESC LIMIT 1",
                [],
                |row| Ok((row.get::<usize, bool>(0)?, row.get::<usize, Vec<u8>>(1)?)),
            )
            .optional()?;
        let (is_root, contribution_or_crs) = match maybe_data {
            None => return Ok(None),
            Some(x) => x,
        };
        let crs = if is_root {
            Phase2RawCeremonyCRS::unchecked_from_protobuf(pb::CeremonyCrs::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
        } else {
            Phase2RawCeremonyContribution::unchecked_from_protobuf(PBContribution::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
            .new_elements()
        };
        Ok(Some(crs))
    }

    pub async fn phase1_commit_contribution(
        &self,
        contributor: Address,
        contribution: Phase1CeremonyContribution,
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let contributor_bytes = contributor.to_vec();
        let hash = contribution.hash().as_ref().to_owned();
        tx.execute(
            "INSERT INTO phase1_contribution_data VALUES(NULL, ?1)",
            (PBContribution::try_from(contribution)?.encode_to_vec(),),
        )?;
        tx.execute(
            "INSERT INTO phase1_contributions VALUES(NULL, 0, ?1, ?2, ?3)",
            (hash, contributor_bytes, current_time_unix()),
        )?;
        tx.commit()?;
        Ok(())
    }

    pub async fn phase2_commit_contribution(
        &self,
        contributor: Address,
        contribution: Phase2CeremonyContribution,
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let contributor_bytes = contributor.to_vec();
        let hash = contribution.hash().as_ref().to_owned();
        tx.execute(
            "INSERT INTO phase2_contribution_data VALUES(NULL, ?1)",
            (PBContribution::try_from(contribution)?.encode_to_vec(),),
        )?;
        tx.execute(
            "INSERT INTO phase2_contributions VALUES(NULL, 0, ?1, ?2, ?3)",
            (hash, contributor_bytes, current_time_unix()),
        )?;
        tx.commit()?;
        Ok(())
    }

    pub async fn current_slot(&self, marker: PhaseMarker) -> Result<u64> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let query = match marker {
            PhaseMarker::P1 => "SELECT MAX(slot) from phase1_contributions",
            PhaseMarker::P2 => "SELECT MAX(slot) from phase2_contributions",
        };
        let out = tx
            .query_row(query, [], |row| row.get::<usize, Option<u64>>(0))?
            .unwrap_or(0);
        Ok(out)
    }

    /// Get Phase 1 root.
    #[allow(dead_code)]
    pub async fn phase1_root(&self) -> Result<Phase1CeremonyCRS> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let data = tx.query_row(
            "SELECT p1_data.contribution_or_crs
 FROM phase1_contribution_data AS p1_data
 JOIN phase1_contributions AS p1 ON p1_data.slot = p1.slot
 WHERE p1.is_root LIMIT 1",
            [],
            |row| row.get::<usize, Vec<u8>>(0),
        )?;
        Ok(
            Phase1RawCeremonyCRS::unchecked_from_protobuf(pb::CeremonyCrs::decode(
                data.as_slice(),
            )?)?
            .assume_valid(),
        )
    }

    /// Get the hash, timestamp, short address of the last N contributors from the database.
    pub async fn last_n_contributors(
        &self,
        marker: PhaseMarker,
        n: u64,
    ) -> Result<Vec<(u64, String, String, String)>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let query = match marker {
            PhaseMarker::P1 =>
                "SELECT slot, hash, time, address from phase1_contributions WHERE address IS NOT NULL ORDER BY slot DESC LIMIT ?1",
            PhaseMarker::P2 =>
                "SELECT slot, hash, time, address from phase2_contributions WHERE address IS NOT NULL ORDER BY slot DESC LIMIT ?1",
        };

        let mut out = Vec::new();
        let mut stmt = tx.prepare(query)?;
        let mut rows = stmt.query([n])?;
        while let Some(row) = rows.next()? {
            let slot: u64 = row.get(0)?;
            let hash_bytes: Vec<u8> = row.get(1)?;
            let unix_timestamp: u64 = row.get(2)?;
            // Convert unix timestamp to date time
            let date_time =
                chrono::DateTime::from_timestamp(unix_timestamp as i64, 0).unwrap_or_default();
            let hash: String = hex::encode_upper(hash_bytes);
            let address_bytes: Vec<u8> = row.get(3)?;
            let address: Address = address_bytes.try_into()?;
            out.push((slot, hash, date_time.to_string(), format!("{}", address)));
        }

        Ok(out)
    }

    /// Get Phase 2 root.
    pub async fn phase2_root(&self) -> Result<Phase2CeremonyCRS> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let data = tx.query_row(
            "SELECT p2_data.contribution_or_crs
 FROM phase2_contribution_data AS p2_data
 JOIN phase2_contributions AS p2 ON p2_data.slot = p2.slot
 WHERE p2.is_root LIMIT 1",
            [],
            |row| row.get::<usize, Vec<u8>>(0),
        )?;
        Ok(
            Phase2RawCeremonyCRS::unchecked_from_protobuf(pb::CeremonyCrs::decode(
                data.as_slice(),
            )?)?
            .assume_valid(),
        )
    }

    pub async fn transition_extra_information(
        &self,
    ) -> Result<Option<AllExtraTransitionInformation>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let maybe_data = tx
            .query_row("SELECT data FROM transition_aux WHERE id = 0", [], |row| {
                row.get::<usize, Vec<u8>>(0)
            })
            .optional()?;
        if let Some(data) = maybe_data {
            Ok(Some(AllExtraTransitionInformation::from_bytes(&data)?))
        } else {
            Ok(None)
        }
    }
}
