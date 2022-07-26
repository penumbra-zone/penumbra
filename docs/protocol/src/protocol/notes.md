# Notes

In Penumbra, value is associated with notes. Once a note is included in a block, it is a valid receipt of value that can later be redeemed. Users can spend their existing notes if they hold the associated spend key and the note has not already been spent. They can transmit value by creating a new note (or "output") for another user. Privacy is achieved by encrypting the notes such that a validator or other observer only sees ciphertext, and does not learn the value or recipient of the note.

The note and its contents are described in further detail in [*Note Plaintexts*](./notes/note_plaintexts.md). Note commitments are described in [*Note Commitments*](./notes/note_commitments.md).