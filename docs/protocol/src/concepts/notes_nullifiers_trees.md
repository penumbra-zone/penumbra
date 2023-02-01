# Notes, Nullifiers, and Trees

Transparent blockchains operate as follows: all participants maintain a copy of
the (consensus-determined) application state. Transactions modify the
application state directly, and participants check that the state changes are
allowed by the application rules before coming to consensus on them.

On a shielded blockchain like Penumbra, however, the state is fragmented across
all users of the application, as each user has a view only of their "local"
portion of the application state recording their funds. Transactions update a
user's state fragments privately, and use a zero-knowledge proof to prove to all
other participants that the update was allowed by the application rules.

Penumbra's transaction model is derived from the Zcash shielded transaction
design, with [modifications][multi_asset] to support multiple asset types, and
several extensions to support additional functionality.  The Zcash model is in
turn derived from Bitcoin's unspent transaction output (UTXO) model, in which
value is recorded in transaction outputs that record the conditions under which
they can be spent.

In Penumbra, value is recorded in *notes*, which function similarly to UTXOs.
Each note specifies (either directly or indirectly) a *type* of value, an
*amount* of value of that type, a *spending key* that authorizes spending the
note's value, and a unique *nullifier* derived from the note's contents.

However, unlike UTXOs, notes are not recorded as part of the public chain state.
Instead, the chain contains a *state commitment tree*, an incremental Merkle tree
containing (public) commitments to (private) notes.  Creating a note involves
creating a new note commitment, and proving that it commits to a valid note.
Spending a note involves proving that the spent note was previously included in
the state commitment tree, using the spending key to demonstrate spend
authorization, and revealing the nullifier, which prevents the same note from
being spent twice.

[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://docs.cosmos.network/master/architecture/adr-001-coin-source-tracing.html
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
