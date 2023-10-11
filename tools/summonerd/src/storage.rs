use anyhow::Result;
use camino::Utf8Path;
use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proof_setup::all::{
    Phase1CeremonyCRS, Phase1RawCeremonyCRS, Phase2CeremonyCRS, Phase2CeremonyContribution,
    Phase2RawCeremonyCRS, Phase2RawCeremonyContribution,
};
use penumbra_proto::{
    penumbra::tools::summoning::v1alpha1::{
        self as pb, participate_request::Contribution as PBContribution,
    },
    Message,
};
use r2d2_sqlite::{
    rusqlite::{OpenFlags, OptionalExtension},
    SqliteConnectionManager,
};
use tokio::task::spawn_blocking;

use crate::penumbra_knower::PenumbraKnower;

const MIN_BID_AMOUNT_U64: u64 = 1u64;
const MAX_STRIKES: u64 = 3u64;

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
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(
        storage_path: impl AsRef<Utf8Path>,
        phase_1_root: Phase1CeremonyCRS,
    ) -> anyhow::Result<Self> {
        if storage_path.as_ref().exists() {
            return Self::load(storage_path, phase_1_root).await;
        }

        Self::initialize(storage_path, phase_1_root).await
    }

    pub async fn initialize(
        storage_path: impl AsRef<Utf8Path>,
        phase_1_root: Phase1CeremonyCRS,
    ) -> anyhow::Result<Self> {
        // Connect to the database (or create it)
        let pool = Self::connect(storage_path)?;

        spawn_blocking(move || {
            // In one database transaction, populate everything
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Create the tables
            tx.execute_batch(include_str!("storage/schema.sql"))?;

            tx.execute(
                "INSERT INTO phase1_contributions VALUES (0, 1, ?1, NULL)",
                [pb::CeremonyCrs::try_from(phase_1_root)?.encode_to_vec()],
            )?;
            // TODO(jen): Transition between phase 1 and phase 2, storing deets in the database
            // using `phase_1_root`
            let phase_2_root = Phase2CeremonyCRS::root()?;
            tx.execute(
                "INSERT INTO phase2_contributions VALUES (0, 1, ?1, NULL)",
                [pb::CeremonyCrs::try_from(phase_2_root)?.encode_to_vec()],
            )?;

            tx.commit()?;

            Ok(Storage { pool })
        })
        .await?
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

    pub async fn load(
        path: impl AsRef<Utf8Path>,
        phase_1_root: Phase1CeremonyCRS,
    ) -> anyhow::Result<Self> {
        let storage = Self {
            pool: Self::connect(path)?,
        };

        let current_phase_1_root = storage.phase_1_root().await?;
        if current_phase_1_root != phase_1_root {
            anyhow::bail!(
                "Phase 1 root in database ({:?}) does not match expected root ({:?})",
                current_phase_1_root,
                phase_1_root
            );
        }

        Ok(storage)
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
                |row| Ok(row.get::<usize, u64>(0)?),
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
    ) -> Result<ContributionAllowed> {
        // Criteria:
        // - Bid more than min amount
        // - Hasn't already contributed
        // - Not banned
        let amount = knower.total_amount_sent_to_me(&address).await?;
        if amount < Amount::from(MIN_BID_AMOUNT_U64) {
            return Ok(ContributionAllowed::DidntBidEnough(amount));
        }
        let has_contributed = {
            let mut conn = self.pool.get()?;
            let tx = conn.transaction()?;
            tx.query_row(
                "SELECT 1 FROM phase2_contributions WHERE address = ?1",
                [address.to_vec()],
                |_| Ok(()),
            )
            .optional()?
            .is_some()
        };
        if has_contributed {
            return Ok(ContributionAllowed::AlreadyContributed);
        }
        if self.get_strikes(address).await? >= MAX_STRIKES {
            return Ok(ContributionAllowed::Banned);
        }
        Ok(ContributionAllowed::Yes(amount))
    }

    pub async fn current_crs(&self) -> Result<Phase2CeremonyCRS> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let (is_root, contribution_or_crs) = tx.query_row(
            "SELECT is_root, contribution_or_crs FROM phase2_contributions ORDER BY slot DESC LIMIT 1",
            [],
            |row| Ok((row.get::<usize, bool>(0)?, row.get::<usize, Vec<u8>>(1)?)),
        )?;
        let crs = if is_root {
            Phase2RawCeremonyCRS::try_from(pb::CeremonyCrs::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
        } else {
            Phase2RawCeremonyContribution::try_from(PBContribution::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
            .new_elements()
        };
        Ok(crs)
    }

    pub async fn commit_contribution(
        &self,
        contributor: Address,
        contribution: Phase2CeremonyContribution,
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let contributor_bytes = contributor.to_vec();
        tx.execute(
            "INSERT INTO phase2_contributions VALUES(NULL, 0, ?1, ?2)",
            [
                PBContribution::try_from(contribution)?.encode_to_vec(),
                contributor_bytes,
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub async fn current_slot(&self) -> Result<u64> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let out = tx
            .query_row("SELECT MAX(slot) FROM phase2_contributions", [], |row| {
                row.get::<usize, Option<u64>>(0)
            })?
            .unwrap_or(0);
        Ok(out)
    }

    /// Get Phase 1 root.
    pub async fn phase_1_root(&self) -> Result<Phase1CeremonyCRS> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let data = tx.query_row(
            "SELECT contribution_or_crs FROM phase1_contributions WHERE is_root LIMIT 1",
            [],
            |row| row.get::<usize, Vec<u8>>(0),
        )?;
        Ok(
            Phase1RawCeremonyCRS::try_from(pb::CeremonyCrs::decode(data.as_slice())?)?
                .assume_valid(),
        )
    }

    /// Get Phase 2 root.
    pub async fn phase_2_root(&self) -> Result<Phase2CeremonyCRS> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let data = tx.query_row(
            "SELECT contribution_or_crs FROM phase2_contributions WHERE is_root LIMIT 1",
            [],
            |row| row.get::<usize, Vec<u8>>(0),
        )?;
        Ok(
            Phase2RawCeremonyCRS::try_from(pb::CeremonyCrs::decode(data.as_slice())?)?
                .assume_valid(),
        )
    }
}
