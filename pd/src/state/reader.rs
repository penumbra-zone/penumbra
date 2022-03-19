use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, VecDeque},
    pin::Pin,
    str::FromStr,
};

use anyhow::{Context, Result};
use async_stream::try_stream;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{asset, note, Address, FieldExt, Fq, Nullifier};
use penumbra_proto::{
    chain,
    light_wallet::{CompactBlock, StateFragment},
    thin_wallet::{Asset, TransactionDetail},
    Protobuf,
};
use penumbra_stake::{
    BaseRateData, Epoch, FundingStream, FundingStreams, IdentityKey, RateData, RateDataById,
    Validator, ValidatorDefinition, ValidatorInfo, ValidatorState, ValidatorStateName,
    ValidatorStatus,
};
use sqlx::{query, query_as, Pool, Postgres};
use tendermint::block;
use tokio::sync::watch;
use tracing::instrument;

use crate::{db::schema, genesis, pd_metrics::MetricsData, verify::NoteData};

#[derive(Debug, Clone)]
pub struct Reader {
    pub(super) pool: Pool<Postgres>,
    //pub(super) tmp: evmap::ReadHandle<&'static str, String>,
    pub(super) chain_params_rx: watch::Receiver<ChainParams>,
    pub(super) height_rx: watch::Receiver<block::Height>,
    pub(super) next_rate_data_rx: watch::Receiver<RateDataById>,
    pub(super) valid_anchors_rx: watch::Receiver<VecDeque<penumbra_tct::Root>>,
}

impl Reader {
    /// Returns a borrowed [`watch::Receiver`] for the latest [`ChainParams`].
    ///
    /// This receiver can be used to access an in-memory copy of the latest data
    /// without accessing the database, but note the warning on
    /// [`watch::Receiver::borrow`] about potential deadlocks.
    pub fn chain_params_rx(&self) -> &watch::Receiver<ChainParams> {
        &self.chain_params_rx
    }

    /// Returns a borrowed [`watch::Receiver`] for the latest [`block::Height`].
    ///
    /// This receiver can be used to access an in-memory copy of the latest data
    /// without accessing the database, but note the warning on
    /// [`watch::Receiver::borrow`] about potential deadlocks.
    pub fn height_rx(&self) -> &watch::Receiver<block::Height> {
        &self.height_rx
    }

    /// Returns a borrowed [`watch::Receiver`] for the latest [`RateDataById`].
    ///
    /// This receiver can be used to access an in-memory copy of the latest data
    /// without accessing the database, but note the warning on
    /// [`watch::Receiver::borrow`] about potential deadlocks.
    pub fn next_rate_data_rx(&self) -> &watch::Receiver<RateDataById> {
        &self.next_rate_data_rx
    }

    /// Returns a borrowed [`watch::Receiver`] for the latest set of valid anchors.
    ///
    /// This receiver can be used to access an in-memory copy of the latest data
    /// without accessing the database, but note the warning on
    /// [`watch::Receiver::borrow`] about potential deadlocks.
    pub fn valid_anchors_rx(&self) -> &watch::Receiver<VecDeque<penumbra_tct::Root>> {
        &self.valid_anchors_rx
    }

    /// Retrieve a nullifier if it exists.
    pub async fn nullifier(&self, nullifier: Nullifier) -> Result<Option<schema::NullifiersRow>> {
        let mut conn = self.pool.acquire().await?;
        let nullifier_row = query!(
            r#"SELECT height FROM nullifiers WHERE nullifier = $1 LIMIT 1"#,
            &<[u8; 32]>::from(nullifier.clone())[..]
        )
        .fetch_optional(&mut conn)
        .await?
        .map(|row| schema::NullifiersRow {
            nullifier,
            height: row.height,
        });

        Ok(nullifier_row)
    }

    /// Retrieve the current note commitment tree.
    pub async fn note_commitment_tree(&self) -> Result<penumbra_tct::Eternity> {
        let mut conn = self.pool.acquire().await?;
        let note_commitment_tree = if let Some(schema::BlobsRow { data, .. }) = query_as!(
            schema::BlobsRow,
            "SELECT id, data FROM blobs WHERE id = 'nct';"
        )
        .fetch_optional(&mut conn)
        .await?
        {
            bincode::deserialize(&data).context("Could not parse saved note commitment tree")?
        } else {
            penumbra_tct::Eternity::new()
        };

        Ok(note_commitment_tree)
    }

    /// Returns statistics for updating the metrics dashboard.
    pub async fn metrics(&self) -> Result<MetricsData> {
        let mut conn = self.pool.acquire().await?;

        let row = query!(
            "
            WITH a AS
            (SELECT COUNT(*) AS nullifier_count FROM nullifiers),
            b AS
            (SELECT COUNT(*) AS note_count FROM notes)
            SELECT nullifier_count, note_count FROM a, b
            "
        )
        .fetch_one(&mut conn)
        .await?;

        Ok(MetricsData {
            nullifier_count: row.nullifier_count.unwrap_or(0) as u64,
            note_count: row.note_count.unwrap_or(0) as u64,
        })
    }

    /// Returns the intersection of the provided nullifiers with the nullifiers
    /// in the database.
    pub async fn check_nullifiers(
        &self,
        nullifiers: &BTreeSet<Nullifier>,
    ) -> Result<BTreeSet<Nullifier>> {
        // https://github.com/launchbadge/sqlx/blob/master/FAQ.md#how-can-i-do-a-select--where-foo-in--query

        let mut conn = self.pool.acquire().await?;

        let nullifiers = nullifiers
            .iter()
            .map(|nf| nf.to_bytes().to_vec())
            .collect::<Vec<_>>();
        let existing = query!(
            "SELECT nullifier FROM nullifiers WHERE nullifier = ANY($1)",
            &nullifiers[..],
        )
        .fetch_all(&mut conn)
        .await?
        .into_iter()
        .map(|row| {
            row.nullifier
                .as_slice()
                .try_into()
                .expect("db data is valid")
        })
        .collect();

        Ok(existing)
    }

    /// Retrieve the node genesis configuration.
    pub async fn genesis_configuration(&self) -> Result<genesis::AppState> {
        let mut conn = self.pool.acquire().await?;
        let genesis_config = if let Some(schema::BlobsRow { data, .. }) = query_as!(
            schema::BlobsRow,
            "SELECT id, data FROM blobs WHERE id = 'gc';"
        )
        .fetch_optional(&mut conn)
        .await?
        {
            serde_json::from_slice(&data).context("Could not parse saved genesis config")?
        } else {
            // This is only reached on the initial startup.
            // The default value here will be overridden by `InitChain`.
            Default::default()
        };

        Ok(genesis_config)
    }

    /// Retrieve the latest block info, if any.
    pub async fn latest_block_info(&self) -> Result<Option<schema::BlocksRow>> {
        let mut conn = self.pool.acquire().await?;
        let latest = query_as!(
            schema::BlocksRow,
            r#"SELECT height, nct_anchor AS "nct_anchor: penumbra_tct::Root", app_hash FROM blocks ORDER BY height DESC LIMIT 1"#
        )
        .fetch_optional(&mut conn)
        .await?;

        Ok(latest)
    }

    // retrieve the `last` latest node commitment tree anchors from the database
    pub async fn recent_anchors(&self, last: usize) -> Result<VecDeque<penumbra_tct::Root>> {
        let mut conn = self.pool.acquire().await?;
        let anchor_rows = query!(
            r#"SELECT nct_anchor AS "nct_anchor: penumbra_tct::Root" FROM blocks ORDER BY height DESC LIMIT $1"#,
            last as i64,
        )
        .fetch_all(&mut conn)
        .await?;

        let mut nct_vec: VecDeque<penumbra_tct::Root> = VecDeque::new();
        for block in anchor_rows {
            nct_vec.push_back(block.nct_anchor)
        }

        Ok(nct_vec)
    }

    /// Retrieve the latest block height.
    pub fn height(&self) -> block::Height {
        *self.height_rx().borrow()
    }

    /// Retrieve the epoch associated with the latest block height.
    pub fn epoch(&self) -> Epoch {
        let epoch_duration = self.chain_params_rx().borrow().epoch_duration;
        Epoch::from_height(self.height().into(), epoch_duration)
    }

    /// Retrieve the latest apphash.
    pub async fn app_hash(&self) -> Result<Vec<u8>> {
        Ok(self
            .latest_block_info()
            .await?
            .map(|row| row.app_hash)
            .unwrap_or_else(|| vec![0; 32]))
    }

    pub async fn base_rate_data(&self, epoch_index: u64) -> Result<BaseRateData> {
        let mut conn = self.pool.acquire().await?;
        let row = query!(
            "SELECT epoch, base_reward_rate, base_exchange_rate
            FROM base_rates
            WHERE epoch = $1",
            epoch_index as i64,
        )
        .fetch_one(&mut conn)
        .await?;

        Ok(BaseRateData {
            epoch_index: row.epoch as u64,
            base_exchange_rate: row.base_exchange_rate as u64,
            base_reward_rate: row.base_reward_rate as u64,
        })
    }

    pub async fn rate_data(&self, epoch_index: u64) -> Result<Vec<RateData>> {
        let mut conn = self.pool.acquire().await?;
        // Select rate data for the given epoch, or for the most recent epoch with rate data less than or equal
        // to the given epoch.
        let rows = query!(
            "
            SELECT DISTINCT ON (identity_key)
            identity_key,
            epoch,
            validator_reward_rate,
            validator_exchange_rate

            FROM validator_rates
            WHERE epoch <= $1
            ORDER BY identity_key, epoch DESC",
            epoch_index as i64,
        )
        .fetch_all(&mut conn)
        .await?;

        Ok(rows
            .into_iter()
            // this does conversions manually rather than using query_as because of i64/u64 casting
            .map(|row| RateData {
                identity_key: IdentityKey::decode(row.identity_key.as_slice())
                    .expect("db data is valid"),
                epoch_index: row.epoch as u64,
                validator_exchange_rate: row.validator_exchange_rate as u64,
                validator_reward_rate: row.validator_reward_rate as u64,
            })
            .collect())
    }

    pub async fn next_rate_data(&self) -> Result<BTreeMap<IdentityKey, RateData>> {
        let mut conn = self.pool.acquire().await?;
        let rows = query!(
            "SELECT identity_key, epoch, validator_reward_rate, validator_exchange_rate
            FROM validator_rates
            WHERE epoch = (SELECT MAX(epoch) from base_rates)",
        )
        .fetch_all(&mut conn)
        .await?;

        Ok(rows
            .into_iter()
            // this does conversions manually rather than using query_as because of i64/u64 casting
            .map(|row| {
                let identity_key =
                    IdentityKey::decode(row.identity_key.as_slice()).expect("db data is valid");

                (
                    identity_key.clone(),
                    RateData {
                        identity_key,
                        epoch_index: row.epoch as u64,
                        validator_exchange_rate: row.validator_exchange_rate as u64,
                        validator_reward_rate: row.validator_reward_rate as u64,
                    },
                )
            })
            .collect())
    }

    pub async fn funding_streams(
        &self,
        validator_identity_key: IdentityKey,
    ) -> Result<FundingStreams> {
        let mut conn = self.pool.acquire().await?;
        let rows = query!(
            "SELECT * from validator_fundingstreams WHERE identity_key = $1",
            validator_identity_key.encode_to_vec(),
        )
        .fetch_all(&mut conn)
        .await?;

        let mut streams = Vec::new();
        for row in rows.into_iter() {
            let addr = row.address.parse::<Address>()?;

            streams.push(FundingStream {
                address: addr,
                rate_bps: row.rate_bps.try_into()?,
            })
        }

        Ok(FundingStreams::try_from(streams)?)
    }

    /// Fetches the latest validator info.
    ///
    /// If `show_inactive` is set, includes validators with 0 voting power.
    pub async fn validator_info(&self, show_inactive: bool) -> Result<Vec<ValidatorInfo>> {
        let mut conn = self.pool.acquire().await?;

        // This would be clearer if we had two queries, but then the generated type of `rows`
        // will be different, forcing duplication of the entire function.
        let power_selector = if show_inactive { i64::MIN } else { 0i64 };
        let rows = query!(
                "SELECT
                    validators.identity_key,
                    validators.voting_power,
                    validator_rates.epoch,
                    validator_rates.validator_reward_rate,
                    validator_rates.validator_exchange_rate,
                    validators.validator_state,
                    validators.unbonding_epoch,
                    validators.name,
                    validators.website,
                    validators.description,
                    validators.consensus_key,
                    validators.sequence_number
                FROM (
                    validators INNER JOIN validator_rates ON validators.identity_key = validator_rates.identity_key
                )
                WHERE validator_rates.epoch = (SELECT MAX(epoch) FROM base_rates) AND NOT voting_power = $1",
                power_selector
            )
            .fetch_all(&mut conn)
            .await?;

        rows.into_iter()
            .map(|row| {
                let identity_key =
                    IdentityKey::decode(row.identity_key.as_slice()).expect("db data is valid");
                let consensus_key =
                    tendermint::PublicKey::from_raw_ed25519(row.consensus_key.as_slice())
                        .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey"))?;
                Ok(ValidatorInfo {
                    validator: Validator {
                        identity_key: identity_key.clone(),
                        consensus_key: consensus_key.clone(),
                        name: row.name,
                        website: row.website,
                        description: row.description,
                        // TODO: Implement
                        funding_streams: FundingStreams::new(),
                        sequence_number: row.sequence_number as u32,
                    },
                    status: ValidatorStatus {
                        identity_key: identity_key.clone(),
                        voting_power: row.voting_power as u64,
                        state: ValidatorState::try_from((
                            ValidatorStateName::from_str(&row.validator_state)?,
                            row.unbonding_epoch.map(|i| i as u64),
                        ))?,
                    },
                    rate_data: RateData {
                        identity_key,
                        epoch_index: row.epoch as u64,
                        validator_exchange_rate: row.validator_exchange_rate as u64,
                        validator_reward_rate: row.validator_reward_rate as u64,
                    },
                })
            })
            .collect()
    }

    /// Retrieve a stream of [`CompactBlock`]s for the given (inclusive) range.
    ///
    /// If the range corresponds to blocks that don't exist, the stream will be empty.
    #[instrument(skip(self))]
    pub fn compact_blocks(
        &self,
        start_height: i64,
        end_height: i64,
    ) -> impl Stream<Item = Result<CompactBlock>> + Send + Unpin {
        let pool = self.pool.clone();
        Box::pin(try_stream! {
            let mut nullifiers = query!(
                "SELECT height, nullifier
                    FROM nullifiers
                    WHERE height BETWEEN $1 AND $2
                    ORDER BY height ASC",
                start_height,
                end_height
            )
            .fetch(&pool)
            .peekable();

            let mut fragments = query!(
                "SELECT height, note_commitment, ephemeral_key, encrypted_note
                    FROM notes
                    WHERE height BETWEEN $1 AND $2
                    ORDER BY position ASC",
                start_height,
                end_height
            )
            .fetch(&pool)
            .peekable();

            for height in start_height..=end_height {
                let mut compact_block = CompactBlock {
                    height: height as u64,
                    fragments: vec![],
                    nullifiers: vec![],
                };

                while let Some(row) = Pin::new(&mut nullifiers).peek().await {
                    // Bail out of the loop if the next iteration would be a different height
                    if let Ok(row) = row {
                        if row.height != height {
                            break;
                        }
                    }

                    let row = Pin::new(&mut nullifiers)
                        .next()
                        .await
                        .expect("we already peeked, so there is a next row")?;
                    compact_block.nullifiers.push(row.nullifier.into());
                }

                while let Some(row) = Pin::new(&mut fragments).peek().await {
                    // Bail out of the loop if the next iteration would be a different height
                    if let Ok(row) = row {
                        if row.height != height {
                            break;
                        }
                    }

                    let row = Pin::new(&mut fragments)
                        .next()
                        .await
                        .expect("we already peeked, so there is a next row")?;
                    compact_block.fragments.push(StateFragment {
                        note_commitment: row.note_commitment.into(),
                        ephemeral_key: row.ephemeral_key.into(),
                        encrypted_note: row.encrypted_note.into(),
                    });
                }

                tracing::debug!(
                    ?height,
                    nullifiers_size = compact_block.nullifiers.len(),
                    fragments_size = compact_block.fragments.len(),
                    "yielding compact block"
                );

                yield compact_block;
            }
        })
    }

    /// Retrieve a stream of quarantined notes and their commitments, paired with the validator
    /// identity key with which they are associated.
    ///
    /// If `maximum_unbonding_height` is `Some`, only notes whose unbonding height is less than or
    /// equal to that height will be returned.
    ///
    /// If `validators` is `Some`, only notes which were associated with an undelegation from some
    /// validator in that set will be returned. (This is more efficient than filtering after
    /// receiving he stream, because the database is performing the filtration.)
    pub fn quarantined_notes<'a>(
        &self,
        maximum_unbonding_height: Option<u64>,
        validators: Option<impl IntoIterator<Item = impl Borrow<&'a IdentityKey>>>,
    ) -> impl Stream<Item = Result<(IdentityKey, note::Commitment, NoteData)>> + Send + Unpin + '_
    {
        // Should we list outputs from all validators?
        let all_validators = validators.is_none();

        // If not, what's the list of validator identities (as bytes) to filter for?
        let validator_list = validators
            .map(|v| v.into_iter().map(|i| i.borrow().encode_to_vec()))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        query!(
            "SELECT validator_identity_key, note_commitment, ephemeral_key, encrypted_note, transaction_id
            FROM quarantined_notes
            WHERE
                unbonding_height <= $1 AND
                ($2 OR validator_identity_key = ANY($3))",
            maximum_unbonding_height.unwrap_or(u64::MAX) as i64,
            all_validators,
            &validator_list,
        )
        .fetch(&self.pool)
        .map_err(Into::into)
        .map(|result| {
            result
                .and_then(|row| {
                    Ok::<_, anyhow::Error>((
                        IdentityKey::decode(&*row.validator_identity_key)?,
                        note::Commitment::try_from(&*row.note_commitment)?,
                        NoteData {
                            ephemeral_key: row.ephemeral_key[..].try_into()?,
                            encrypted_note: row.encrypted_note[..].try_into()?,
                            transaction_id: row.transaction_id[..].try_into()?,
                        },
                    ))
                })
                .map_err(Into::into)
        })
    }

    /// Retrieve a stream of quarantined nullifiers, paired with the validator identity key with
    /// which they are associated.
    ///
    /// If `maximum_unbonding_height` is `Some`, only nullifiers whose unbonding height is less than or
    /// equal to that height will be returned.
    ///
    /// If `validators` is `Some`, only nullifiers which were associated with an undelegation from some
    /// validator in that set will be returned. (This is more efficient than filtering after
    /// receiving he stream, because the database is performing the filtration.)
    pub fn quarantined_nullifiers<'a>(
        &self,
        maximum_unbonding_height: Option<u64>,
        validators: Option<impl IntoIterator<Item = impl Borrow<&'a IdentityKey>>>,
    ) -> impl Stream<Item = Result<(IdentityKey, Nullifier)>> + Send + Unpin + '_ {
        // Should we list outputs from all validators?
        let all_validators = validators.is_none();

        // If not, what's the list of validator identities (as bytes) to filter for?
        let validator_list = validators
            .map(|v| v.into_iter().map(|i| i.borrow().encode_to_vec()))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        query!(
            "SELECT validator_identity_key, nullifier
            FROM quarantined_nullifiers
            WHERE
                unbonding_height <= $1 AND
                ($2 OR validator_identity_key = ANY($3))",
            maximum_unbonding_height.unwrap_or(u64::MAX) as i64,
            all_validators,
            &validator_list,
        )
        .fetch(&self.pool)
        .map_err(Into::into)
        .map(|result| {
            result
                .and_then(|row| {
                    Ok::<_, anyhow::Error>((
                        IdentityKey::decode(&*row.validator_identity_key)?,
                        Nullifier::try_from(&row.nullifier[..])?,
                    ))
                })
                .map_err(Into::into)
        })
    }

    /// Retrieve the [`TransactionDetail`] for a given note commitment.
    pub async fn transaction_by_note(&self, note_commitment: Vec<u8>) -> Result<TransactionDetail> {
        let mut conn = self.pool.acquire().await?;

        let row = query!(
            "SELECT transaction_id FROM notes WHERE note_commitment = $1",
            note_commitment
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(TransactionDetail {
            id: row.transaction_id,
        })
    }

    /// Retrieve the [`Asset`] for a given asset ID.
    pub async fn asset_lookup(&self, asset_id: asset::Id) -> Result<Option<chain::AssetInfo>> {
        let mut conn = self.pool.acquire().await?;

        let asset = query!(
            "SELECT denom, asset_id, total_supply FROM assets WHERE asset_id = $1",
            asset_id.to_bytes().to_vec(),
        )
        .fetch_optional(&mut conn)
        .await?;

        let height = self.height();

        // TODO: should we be returning proto types from our state methods, or domain types?
        Ok(asset.map(|asset| {
            let inner = Fq::from_bytes(asset.asset_id.try_into().unwrap())
                .expect("invalid asset id in database");

            chain::AssetInfo {
                denom: Some(
                    asset::REGISTRY
                        .parse_denom(asset.denom.as_str())
                        .unwrap()
                        .into(),
                ),
                asset_id: Some(asset::Id(inner).into()),
                total_supply: asset.total_supply as u64, // postgres only has i64....
                as_of_block_height: u64::from(height),
            }
        }))
    }

    /// Retrieves the entire Asset Registry.
    pub async fn asset_list(&self) -> Result<Vec<Asset>> {
        let mut conn = self.pool.acquire().await?;

        Ok(query!("SELECT denom, asset_id FROM assets")
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(|row| Asset {
                asset_denom: row.denom,
                asset_id: row.asset_id,
            })
            .collect())
    }

    /// Retrieve the delegation changes for the supplied epoch
    /// TODO: should we have a DelegationChanges struct instead of just returning a BTreeMap?
    pub async fn delegation_changes(&self, epoch: u64) -> Result<BTreeMap<IdentityKey, i64>> {
        let mut conn = self.pool.acquire().await?;

        let rows = query!("SELECT validator_identity_key, delegation_change FROM delegation_changes WHERE epoch = $1",
               epoch as i64
            ).fetch_all(&mut conn)
            .await?;

        let mut changes: BTreeMap<IdentityKey, i64> = BTreeMap::new();
        for row in rows {
            let id_key = IdentityKey::decode(row.validator_identity_key.as_slice())?;
            changes.insert(id_key, row.delegation_change);
        }

        Ok(changes)
    }
}
