# Multi-Asset Shielded Pool

Penumbra records value in a single, *multi-asset shielded pool*.  Value is recorded in _notes_, which record a typed quantity of value together with a spending capability describing who controls the value.

However, the notes themselves are never published to the chain.  Instead, the shielded pool records opaque _commitments_ to notes in the [state commitment tree](./sct.md).  To create a new note, a transaction includes an `Output` action, which contains the commitment to the newly created note and a zero-knowledge proof that it was honestly constructed.  To spend an existing note, a transaction includes a `Spend` action, which includes a zero-knowledge proof that the note's commitment was previously included in the state commitment tree -- but without revealing which one.

To prevent double-spending, each note has a unique serial number, called a _nullifier_.  Each `Spend` action reveals the nullifier of the spent note, and the chain checks that the nullifier has not already been revealed in a previous transaction.  Because the nullifier can only be derived using the keys that control the note, third parties cannot link spends and outputs.

The note and its contents are described in further detail in [*Note Plaintexts*](./notes/note_plaintexts.md). Note commitments are described in [*Note Commitments*](./notes/note_commitments.md).