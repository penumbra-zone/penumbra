

# macos 

```
brew install postgresql@16
```
start
```
brew services start postgresql@16
```
create database `testnet_raw`
```
$ psql postgres
psql (16.3 (Homebrew))
Type "help" for help.

postgres=# CREATE DATABASE testnet_raw;
CREATE DATABASE
postgres=#
\q
```
test connection
```
psql "postgresql://localhost:5432/testnet_raw?sslmode=disable"
```
put in comet config.toml
```
[tx_index]
indexer = "psql"
psql-conn = "postgresql://localhost:5432/testnet_raw?sslmode=disable"
```

example data
```
testnet_raw=# SELECT * FROM events JOIN blocks ON events.block_id = blocks.rowid WHERE blocks.height = 426939 ;
 rowid | block_id | tx_id |                            type                             | rowid | height |         chain_id          |          created_at
-------+----------+-------+-------------------------------------------------------------+-------+--------+---------------------------+-------------------------------
 99043 |     7534 |       | block                                                       |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99044 |     7534 |       | penumbra.core.component.sct.v1.EventAnchor                  |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99045 |     7534 |       | penumbra.core.component.sct.v1.EventBlockRoot               |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99046 |     7534 |  4350 | tx                                                          |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99047 |     7534 |  4350 | tx                                                          |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99048 |     7534 |  4350 | penumbra.core.component.shielded_pool.v1.EventSpend         |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99049 |     7534 |  4350 | penumbra.core.component.sct.v1.EventCommitment              |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99050 |     7534 |  4350 | penumbra.core.component.shielded_pool.v1.EventOutput        |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99051 |     7534 |  4350 | penumbra.core.component.sct.v1.EventCommitment              |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99052 |     7534 |  4350 | penumbra.core.component.shielded_pool.v1.EventOutput        |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99053 |     7534 |  4350 | penumbra.core.component.shielded_pool.v1.EventBroadcastClue |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99054 |     7534 |  4350 | penumbra.core.component.shielded_pool.v1.EventBroadcastClue |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99055 |     7534 |  4351 | tx                                                          |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99056 |     7534 |  4351 | tx                                                          |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99057 |     7534 |  4351 | penumbra.core.component.shielded_pool.v1.EventSpend         |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99058 |     7534 |  4351 | penumbra.core.component.sct.v1.EventCommitment              |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99059 |     7534 |  4351 | penumbra.core.component.shielded_pool.v1.EventOutput        |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99060 |     7534 |  4351 | penumbra.core.component.sct.v1.EventCommitment              |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99061 |     7534 |  4351 | penumbra.core.component.shielded_pool.v1.EventOutput        |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99062 |     7534 |  4351 | penumbra.core.component.shielded_pool.v1.EventBroadcastClue |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
 99063 |     7534 |  4351 | penumbra.core.component.shielded_pool.v1.EventBroadcastClue |  7534 | 426939 | penumbra-testnet-deimos-8 | 2024-06-07 01:38:33.440578-04
(21 rows)
```

example invocations
```
cargo run --bin pindexer -- -s "postgresql://localhost:5432/testnet_raw?sslmode=disable" -d "postgresql://localhost:5432/testnet_compiled?sslmode=disable"
```
```
cargo run --example fmd_clues -- -s "postgresql://localhost:5432/testnet_raw?sslmode=disable" -d "postgresql://localhost:5432/testnet_compiled?sslmode=disable"
```