# Indexing ABCI events

The `pd` software emits ABCI events while processing blocks. By default,
these blocks are stored in CometBFT's key-value database locally, but node operators
can opt-in to writing the events to an external PostgreSQL database.

## Configuring a Penumbra node to write events to postgres

1. Create a Postgres database, username, and credentials.
2. Apply the [CometBFT schema] to the database: `psql -d $DATABASE_NAME -f <path/to/schema.sql>`
3. Edit the CometBFT config file, by default located at `~/.penumbra/network_data/node0/cometbft/config/config.toml`,
   specifically its `[tx_index]`, and set:
   1. `indexer = "psql"`
   2. `psql-conn = "<DATABASE_URL>"`
4. Run `pd` and `cometbft` as normal.

The format for `DATABASE_URL` is specified in the [Postgres docs](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING-URIS).
After the node is running, check the logs for errors. Query the database with `SELECT height FROM blocks ORDER BY height DESC LIMIT 10;` and confirm
you're seeing the latest blocks being added to the database.

## Rebuilding an index database from scratch

If you are joining the network after a chain upgrade, the events behind the upgrade boundary
will not be available to your node for syncing while catching up to current height. To emit
historical events, you will need access to archives of CometBFT state created before (each)
planned upgrade. The process then becomes:

1. Restore node state from backup.
2. Ensure you're using the appropriate `pd` and `cometbft` versions for the associated state.
3. Run `pd migrate --ready-to-start` to permit `pd` to start up.
4. Run CometBFT with extra options: `--p2p.pex=false --p2p.seeds='' --moniker archive-node-1`
5. Run `pd` and `cometbft` as normal, taking care to use the appropriate versions.

Then configure another node with indexing support, as described above, and join the second
node to the archive node. As it streams blocks, the ABCI events will be recorded in the database.

[CometBFT schema]: https://github.com/cometbft/cometbft/blob/main/state/indexer/sink/psql/schema.sql
