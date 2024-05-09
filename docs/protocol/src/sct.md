# State Commitment Tree

Penumbra makes an explicit separation between two different kinds of state:

* Public shared state, recorded on-chain in a key-value store;
* Private per-user state, recorded on end-user devices and only committed to on-chain.

Public state is global and mutable, like other blockchains (akin to an "account
model"). Transactions modify the public state as they execute.  Private state is
recorded in immutable, composable _state fragments_ (akin to a "UTXO model").
Transactions privately consume existing state fragments, compose them to
privately create new state fragments, and use zero-knowledge proofs to certify
that the new fragments were created according to the rules of the protocol.

The state commitment tree component manages commitments to private state
fragments. Nodes maintain two data structures, each responsible for one side of
state management:

1. The _state commitment tree_ ingests and records new state commitments as transactions produce new state fragments;
2. The _nullifier set_ is responsible for nullifying existing state commitments as transactions consume state fragments.

To establish that a state fragment was previously validated by the chain, a
client forms a ZK proof that the commitment to that state fragment is included
in the state commitment tree.  This allows private execution to reference prior
state fragments, without revealing _which_ specific state fragment is
referenced.

Private execution involves a role reversal relative to the conventional use of
Merkle trees in blockchains, because it is the _client_ that makes a proof to
the _full node_, rather than the other way around.  This means that every client
must synchronize a local instance of the state commitment tree, in order to have
the data required to form proofs.  Penumbra's state commitment tree is
instantiated using the _tiered commitment tree_ (TCT), an append-only,
ZK-friendly Merkle tree designed primarily for efficient, filterable
synchronization.  We use the term SCT to refer to the part of the protocol that
manages state commitments and TCT to refer to the specific Merkle tree
construction used to instantiate it.  The TCT is described in the [Tiered
Commitment Tree](./sct/tct.md) section.

Establishing that a state fragment was previously validated by the chain is not
enough, however.  Clients also need to establish that the state fragments their
transaction consumes have not _already_ been consumed by another transaction, to
prevent double-spend attacks.

This is accomplished by the use of _nullifiers_, described in detail in the
[Nullifiers](./sct/nullifiers.md) section.  Each state commitment has an
associated nullifier.  Transactions reveal the nullifiers of state fragments
they consume, and the chain checks that they have not been previously revealed.
Nullifiers can only be derived by the actor with authority over the state
fragment, so the creation of a state fragment (revealing its state commitment)
and the consumption of that state fragment (revealing its nullifier) are
publicly unlinkable. In practice, the nullifier set is stored in the public
state using the [Jellyfish Merkle Tree (JMT)](https://diem-developers-components.netlify.app/papers/jellyfish-merkle-tree/2021-01-14.pdf).
