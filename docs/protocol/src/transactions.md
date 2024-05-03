# Transaction Model

A Penumbra transaction is a bundle of _actions_ that effect changes to the chain
state, together with additional data controlling how those actions are executed
or providing additional metadata. All actions in the transaction are executed together, and the transaction succeeds or fails atomically. A transaction body has four parts:

1. A list of actions effecting changes to the chain state;
2. A set of `TransactionParameters` describing conditions under which the actions may be executed;
3. An optional set of `DetectionData` that helps third-party servers detect transactions using [Fuzzy Message Detection](./crypto/fmd.md);
4. An optional `MemoCiphertext` with an [encrypted memo](./transactions/memo.md) visible only to the sender and receiver(s) of the transaction.

The [Transaction Signing](./transactions/signing.md) section describes transaction authorization.

## Actions and Value Balance

The primary content of a transaction is its list of actions. Each action is executed in sequence, and effects changes to the chain state.

Crucially, each action makes a shielded contribution to the transaction's value
balance by means of a _balance commitment_, using the commitment scheme for asset values described in detail in the [Asset Model](./assets.md).

Some actions, like a `Spend`, consume state fragments from the chain and release
value into the transaction, while others, like `Output`, consume value from the
transaction and record it in the state. And actions like `Delegate` consume one
type of value (the staking token) and release another type of value (the
delegation token).

The chain requires that transactions do not create or destroy value.  To
accomplish conservation of value, the _binding signature_ proves that the
transaction's value balance, summed up over all actions, is zero.  This
construction works as follows.  We'd like to be able to prove that a certain
balance commitment $C$ is a commitment to $0$.  One way to do this would be to
prove knowledge of an opening to the commitment, i.e., producing $\widetilde{v}$
such that $$C = [\widetilde{v}] \widetilde{V} = \operatorname{Commit}(0,
\widetilde{v}).$$  But this is exactly what it means to create a Schnorr
signature for the verification key $C$, because a Schnorr signature is a proof
of knowledge of the signing key in the context of the message.

Therefore, we can prove that a balance commitment is a commitment to $0$ by
treating it as a `decaf377-rdsa` verification key and using the corresponding
signing key (the blinding factor) to sign a message.  This also gives a way to
bind balance commitments to a particular context (e.g., a transaction), by using a
hash of the transaction as the message to be signed, ensuring that actions
cannot be replayed across transactions without knowledge of their contents.

The balance commitment mechanism is essential to security. It is what "glues"
the validity of the local state transitions provided by each action's proof
statement into validity of the global state transition.

A complete list of actions and a summary of their effects on the chain state and the transaction's value balance is provided in the [Action Reference](./transactions/actions.md).
