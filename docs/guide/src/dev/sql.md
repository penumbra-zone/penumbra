# Working with SQLite

The view server uses [SQLite3](https://sqlite.org) to store client state locally.
During debugging, you may wish to interact with the sqlite db directly.
To do so:

```
$ sqlite3 ~/.local/share/pcli/pcli-view.sqlite
sqlite> PRAGMA table_info(tx);
0|tx_hash|BLOB|1||1
1|tx_bytes|BLOB|1||0
2|block_height|BIGINT|1||0
3|return_address|BLOB|0||0

sqlite> SELECT json_object('tx_hash', quote(tx_hash)) FROM tx;
{"tx_hash":"X'14672F89F5B197C45D85189AE5A24C47F4F22417B5DC33B50FD263DE2E10BFD3'"}
{"tx_hash":"X'DCA7DB158D93372A0ED335924B05946F336C28BC76AC90BABEF4E1466022D2D2'"}
```

Note that because binary data is stored directly in the db (see `BLOB` in pragma),
you'll need to decode the blob as a JSON object to get readable info.

## Viewing IBC assets

To list assets that have been transferred in via IBC, query on the denom for
a prefix of `transfer/`:

```
sqlite> SELECT denom, json_object('asset_id', quote(asset_id)) FROM assets WHERE denom LIKE 'transfer/%' ;
transfer/channel-0/uosmo|{"asset_id":"X'8C8A30604A6832BF8B418AA30D512740EEA1CB36FDADA5716CED462F20F19612'"}
```
