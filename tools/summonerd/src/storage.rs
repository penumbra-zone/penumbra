use anyhow::Result;
use camino::Utf8Path;
use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proof_setup::all::{
    AllExtraTransitionInformation, Phase1CeremonyCRS, Phase1RawCeremonyCRS,
    Phase1RawCeremonyContribution, Phase2CeremonyCRS, Phase2CeremonyContribution,
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
    pub async fn load_or_initialize(storage_path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        if storage_path.as_ref().exists() {
            return Self::load(storage_path).await;
        }

        Self::initialize(storage_path).await
    }

    /// Initialize creates the database, but does not insert anything into it.
    async fn initialize(storage_path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        // Connect to the database (or create it)
        let pool = Self::connect(storage_path)?;

        spawn_blocking(move || {
            // In one database transaction, populate everything
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Create the tables
            tx.execute_batch(include_str!("storage/schema.sql"))?;

            tx.commit()?;

            Ok(Storage { pool })
        })
        .await?
    }

    async fn load(path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        let storage = Self {
            pool: Self::connect(path)?,
        };

        Ok(storage)
    }

    /// Set the root we need for phase1.
    pub async fn set_root(&mut self, phase_1_root: Phase1CeremonyCRS) -> anyhow::Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO phase1_contributions VALUES (0, 1, ?1, NULL)",
            [pb::CeremonyCrs::try_from(phase_1_root)?.encode_to_vec()],
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
            "INSERT INTO phase2_contributions VALUES (0, 1, ?1, NULL)",
            [pb::CeremonyCrs::try_from(phase_2_root)?.encode_to_vec()],
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

    pub async fn phase1_current_crs(&self) -> Result<Option<Phase1CeremonyCRS>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let maybe_data = tx.query_row(
            "SELECT is_root, contribution_or_crs FROM phase1_contributions ORDER BY slot DESC LIMIT 1",
            [],
            |row| Ok((row.get::<usize, bool>(0)?, row.get::<usize, Vec<u8>>(1)?)),
        ).optional()?;
        let (is_root, contribution_or_crs) = match maybe_data {
            None => return Ok(None),
            Some(x) => x,
        };
        let crs = if is_root {
            Phase1RawCeremonyCRS::try_from(pb::CeremonyCrs::decode(
                contribution_or_crs.as_slice(),
            )?)?
            .assume_valid()
        } else {
            Phase1RawCeremonyContribution::try_from(PBContribution::decode(
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
        let maybe_data = tx.query_row(
            "SELECT is_root, contribution_or_crs FROM phase2_contributions ORDER BY slot DESC LIMIT 1",
            [],
            |row| Ok((row.get::<usize, bool>(0)?, row.get::<usize, Vec<u8>>(1)?)),
        ).optional()?;
        let (is_root, contribution_or_crs) = match maybe_data {
            None => return Ok(None),
            Some(x) => x,
        };
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
        Ok(Some(crs))
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
    #[allow(dead_code)]
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

    pub async fn transition_extra_information(
        &self,
    ) -> Result<Option<AllExtraTransitionInformation>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let maybe_data = tx
            .query_row("SELECT data FROM transition_aux WHERE id = 0", [], |row| {
                Ok(row.get::<usize, Vec<u8>>(0)?)
            })
            .optional()?;
        if let Some(data) = maybe_data {
            Ok(Some(AllExtraTransitionInformation::from_bytes(&data)?))
        } else {
            return Ok(None);
        }
    }
}
