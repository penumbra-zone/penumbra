use std::{
    collections::{BTreeMap, VecDeque},
    pin::Pin,
};

use anyhow::{Context, Result};
use async_stream::try_stream;
use futures::{
    future::BoxFuture,
    stream::{Stream, StreamExt},
};
use jmt::{
    hash::{CryptoHash, CryptoHasher, HashValue, TestOnlyHasher},
    node_type::{LeafNode, Node, NodeKey},
    restore::JellyfishMerkleRestore,
    JellyfishMerkleTree, NodeBatch, TreeReaderAsync, TreeWriterAsync, Value,
};
use penumbra_crypto::{
    asset,
    merkle::{self, NoteCommitmentTree, TreeExt},
    Address, FieldExt, Fq, Nullifier,
};
use penumbra_proto::{
    chain,
    light_wallet::{CompactBlock, StateFragment},
    thin_wallet::{Asset, TransactionDetail},
    Protobuf,
};
use penumbra_stake::{
    BaseRateData, FundingStream, IdentityKey, RateData, Validator, ValidatorInfo, ValidatorStatus,
};
use sqlx::{postgres::PgPoolOptions, query, query_as, Pool, Postgres};
use tendermint::block;
use tracing::instrument;

use crate::{db::schema, genesis, PendingBlock};

#[derive(Debug, Clone)]
pub struct State {
    pool: Pool<Postgres>,
}

// NOTE: because the state has a Postgres connection pool and supports shared
// access, all methods (including write methods!) take &self, not &mut self.
//
// Writes to the database should *only* happen in `commit_genesis` (called once
// on init) and `commit_block`.

impl State {
    /// Connect to the database with the given `uri`.
    #[instrument]
    pub async fn connect(uri: &str) -> Result<Self> {
        tracing::info!("connecting to postgres");
        let pool = PgPoolOptions::new().max_connections(4).connect(uri).await?;

        tracing::info!("running migrations");
        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("finished initializing state");

        let start = std::time::Instant::now();
        let mut conn = pool.acquire().await?;
        query!("SELECT true").fetch_all(&mut conn).await?;
        let end = start.elapsed();
        let ms = (end.as_micros() as f64) / 1000.;
        tracing::info!("no-op query took {}ms ({:.0} per second)", ms, 1000. / ms);

        Ok(State { pool })
    }

    /// Commits the genesis config to the database, prior to the first block commit.
    pub async fn commit_genesis(&self, genesis_config: &genesis::AppState) -> Result<()> {
        let mut dbtx = self.pool.begin().await?;

        let genesis_bytes = serde_json::to_vec(&genesis_config)?;

        // ON CONFLICT is excluded here so that an error is raised
        // if genesis config is attempted to be set more than once
        query!(
            r#"
            INSERT INTO blobs (id, data) VALUES ('gc', $1)
            "#,
            &genesis_bytes[..]
        )
        .execute(&mut dbtx)
        .await?;

        query!("INSERT INTO jmt (value) VALUES ($1)", &genesis_bytes[..])
            .execute(&mut dbtx)
            .await?;

        // Delegations require knowing the rates for the next epoch, so
        // pre-populate with 0 reward => exchange rate 1 for the current
        // (index 0) and next (index 1) epochs.
        for epoch in [0, 1] {
            query!(
                "INSERT INTO base_rates (
                epoch,
                base_reward_rate,
                base_exchange_rate
            ) VALUES ($1, $2, $3)",
                epoch,
                0,
                1_0000_0000
            )
            .execute(&mut dbtx)
            .await?;
        }

        for genesis::ValidatorPower { validator, power } in &genesis_config.validators {
            query!(
                "INSERT INTO validators (
                    identity_key,
                    consensus_key,
                    sequence_number,
                    validator_data,
                    voting_power
                ) VALUES ($1, $2, $3, $4, $5)",
                validator.identity_key.encode_to_vec(),
                validator.consensus_key.to_bytes(),
                validator.sequence_number as i64,
                validator.encode_to_vec(),
                power.value() as i64,
            )
            .execute(&mut dbtx)
            .await?;

            for FundingStream { address, rate_bps } in &validator.funding_streams {
                query!(
                    "INSERT INTO validator_fundingstreams (
                        identity_key,
                        address,
                        rate_bps
                    ) VALUES ($1, $2, $3)",
                    validator.identity_key.encode_to_vec(),
                    address.to_string(),
                    *rate_bps as i32,
                )
                .execute(&mut dbtx)
                .await?;
            }

            // The initial voting power is set from the genesis configuration,
            // but later, it's recomputed based on the size of each validator's
            // delegation pool.  Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current (index 0) and next (index 1) epochs.
            for epoch in [0, 1] {
                query!(
                    "INSERT INTO validator_rates (
                    identity_key,
                    epoch,
                    validator_reward_rate,
                    validator_exchange_rate
                ) VALUES ($1, $2, $3, $4)",
                    validator.identity_key.encode_to_vec(),
                    epoch,
                    0,
                    1_00000000i64, // 1 represented as 1e8
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        dbtx.commit().await.map_err(Into::into)
    }

    pub async fn commit_block(&self, block: PendingBlock) -> Result<()> {
        let mut dbtx = self.pool.begin().await?;

        let nct_anchor = block.note_commitment_tree.root2();
        // TODO: work out what other stuff to put in apphashes
        let app_hash = nct_anchor.to_bytes();
        let height = block.height.expect("height must be set");

        let nct_bytes = bincode::serialize(&block.note_commitment_tree)?;

        // TODO: batch these queries?

        query!(
            r#"
            INSERT INTO blobs (id, data) VALUES ('nct', $1)
            ON CONFLICT (id) DO UPDATE SET data = $1
            "#,
            &nct_bytes[..]
        )
        .execute(&mut dbtx)
        .await?;

        //insert block state into JMT

        let (_, tree_update_batch) = jmt::JellyfishMerkleTree::new(self)
            .put_value_set(
                vec![(HashValue::sha3_256_of(b"nct"), nct_anchor.clone())],
                height,
            )
            .await?;

        //insert block state into JMT's backing postgres table

        DbTx(&mut dbtx)
            .write_node_batch(&tree_update_batch.node_batch)
            .await?;

        query!(
            "INSERT INTO blocks (height, nct_anchor, app_hash) VALUES ($1, $2, $3)",
            height as i64,
            &nct_anchor.to_bytes()[..],
            &app_hash[..]
        )
        .execute(&mut dbtx)
        .await?;

        // Add newly created notes into the chain state.
        for (note_commitment, positioned_note) in block.notes.into_iter() {
            query!(
                r#"
                INSERT INTO notes (
                    note_commitment,
                    ephemeral_key,
                    encrypted_note,
                    transaction_id,
                    position,
                    height
                ) VALUES ($1, $2, $3, $4, $5, $6)"#,
                &<[u8; 32]>::from(note_commitment)[..],
                &positioned_note.data.ephemeral_key.0[..],
                &positioned_note.data.encrypted_note[..],
                &positioned_note.data.transaction_id[..],
                positioned_note.position as i64,
                height as i64,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Mark spent notes as spent.
        for nullifier in block.spent_nullifiers.into_iter() {
            query!(
                "INSERT INTO nullifiers VALUES ($1, $2)",
                &<[u8; 32]>::from(nullifier)[..],
                height as i64,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Track the net change in delegations in this block.
        let epoch_index = block.epoch.unwrap().index;
        for (identity_key, delegation_change) in block.delegation_changes {
            query!(
                "INSERT INTO delegation_changes VALUES ($1, $2, $3)",
                identity_key.encode_to_vec(),
                epoch_index as i64,
                delegation_change
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Save any new assets found in the block to the asset registry.
        for (id, asset) in block.supply_updates {
            query!(
                r#"INSERT INTO assets (asset_id, denom, total_supply) VALUES ($1, $2, $3) ON CONFLICT (asset_id) DO UPDATE SET denom=$2, total_supply=$3"#,
                &id.to_bytes()[..],
                asset.0.to_string(),
                asset.1 as i64
            )
            .execute(&mut dbtx)
            .await?;
        }

        if let (Some(base_rate_data), Some(rate_data)) = (block.next_base_rate, block.next_rates) {
            query!(
                "INSERT INTO base_rates VALUES ($1, $2, $3)",
                base_rate_data.epoch_index as i64,
                base_rate_data.base_reward_rate as i64,
                base_rate_data.base_exchange_rate as i64,
            )
            .execute(&mut dbtx)
            .await?;

            for rate in rate_data {
                query!(
                    "INSERT INTO validator_rates VALUES ($1, $2, $3, $4)",
                    rate.identity_key.encode_to_vec(),
                    rate.epoch_index as i64,
                    rate.validator_reward_rate as i64,
                    rate.validator_exchange_rate as i64,
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        if let Some(validator_statuses) = block.next_validator_statuses {
            for status in validator_statuses {
                query!(
                    "UPDATE validators SET voting_power=$1 WHERE identity_key = $2",
                    status.voting_power as i64,
                    status.identity_key.encode_to_vec(),
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        dbtx.commit().await.map_err(Into::into)
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
    pub async fn note_commitment_tree(&self) -> Result<NoteCommitmentTree> {
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
            NoteCommitmentTree::new(0)
        };

        Ok(note_commitment_tree)
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
            r#"SELECT height, nct_anchor AS "nct_anchor: merkle::Root", app_hash FROM blocks ORDER BY height DESC LIMIT 1"#
        )
        .fetch_optional(&mut conn)
        .await?;

        Ok(latest)
    }

    // retrieve the `last` latest node commitment tree anchors from the database
    pub async fn recent_anchors(&self, last: usize) -> Result<VecDeque<merkle::Root>> {
        let mut conn = self.pool.acquire().await?;
        let anchor_rows = query!(
            r#"SELECT nct_anchor AS "nct_anchor: merkle::Root" FROM blocks ORDER BY height DESC LIMIT $1"#,
            last as i64,
        )
        .fetch_all(&mut conn)
        .await?;

        let mut nct_vec: VecDeque<merkle::Root> = VecDeque::new();
        for block in anchor_rows {
            nct_vec.push_back(block.nct_anchor)
        }

        Ok(nct_vec)
    }

    /// Retrieve the latest block height.
    pub async fn height(&self) -> Result<block::Height> {
        Ok(self
            .latest_block_info()
            .await?
            .map(|row| row.height)
            .unwrap_or(0)
            .try_into()
            .unwrap())
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
        let rows = query!(
            "SELECT identity_key, epoch, validator_reward_rate, validator_exchange_rate
            FROM validator_rates
            WHERE epoch = $1",
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

    pub async fn funding_streams(
        &self,
        validator_identity_key: IdentityKey,
    ) -> Result<Vec<FundingStream>> {
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

        Ok(streams)
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
                    validators.validator_data 
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
                Ok(ValidatorInfo {
                    validator: Validator::decode(row.validator_data.as_slice())?,
                    status: ValidatorStatus {
                        identity_key: identity_key.clone(),
                        voting_power: row.voting_power as u64,
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
                    height: height as u32,
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

        let height = self.height().await?;

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

struct DbTx<'conn, 'tx>(pub &'tx mut sqlx::Transaction<'conn, Postgres>);

impl<'conn, 'tx, V> TreeWriterAsync<V> for DbTx<'conn, 'tx>
where
    V: Value,
{
    /// Writes a node batch into storage.
    fn write_node_batch<'future, 'a: 'future, 'n: 'future>(
        &'a mut self,
        node_batch: &'n NodeBatch<V>,
    ) -> BoxFuture<'future, Result<()>> {
        Box::pin(async move {
            for (node_key, node) in node_batch.clone() {
                let key_bytes = &node_key.encode()?;
                let value_bytes = &node.encode()?;

                query!(
                    r#"
                INSERT INTO jmt (key, value) VALUES ($1, $2)
                "#,
                    &key_bytes,
                    &value_bytes
                )
                .execute(&mut *self.0)
                .await?;
            }

            Ok(())
        })
    }
}

impl<V: jmt::Value> TreeReaderAsync<V> for State {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option<'future, 'a: 'future, 'n: 'future>(
        &'a self,
        node_key: &'n NodeKey,
    ) -> BoxFuture<'future, Result<Option<Node<V>>>> {
        Box::pin(async {
            let mut conn = self.pool.acquire().await?;

            let value = query!(
                r#"SELECT value FROM jmt WHERE key = $1 LIMIT 1"#,
                &node_key.encode()?
            )
            .fetch_optional(&mut conn)
            .await?;

            let value = match value {
                Some(row) => Some(Node::decode(&row.value)?),
                _ => None,
            };

            Ok(value)
        })
    }

    /// Gets the rightmost leaf. Note that this assumes we are in the process of restoring the tree
    /// and all nodes are at the same version.
    #[allow(clippy::type_complexity)]
    fn get_rightmost_leaf<'future, 'a: 'future>(
        &'a self,
    ) -> BoxFuture<'future, Result<Option<(NodeKey, LeafNode<V>)>>> {
        Box::pin(async {
            let mut conn = self.pool.acquire().await?;

            let value = query!(r#"SELECT key, value FROM jmt ORDER BY key DESC LIMIT 1"#)
                .fetch_optional(&mut conn)
                .await?;

            let value = match value {
                Some(row) => Some((NodeKey::decode(&row.key)?, Node::decode(&row.value)?)),
                _ => None,
            };

            let mut node_key_and_node: Option<(NodeKey, LeafNode<V>)> = None;

            if let Some((key, Node::Leaf(leaf_node))) = value {
                if node_key_and_node.is_none()
                    || leaf_node.account_key() > node_key_and_node.as_ref().unwrap().1.account_key()
                {
                    node_key_and_node.replace((key, leaf_node));
                }
            }

            Ok(node_key_and_node)
        })
    }
}
