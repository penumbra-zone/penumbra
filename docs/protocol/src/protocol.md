# Cryptographic Protocol

This chapter describes the cryptographic design of the Penumbra protocol,
described in more detail in the following sections:

- The [Addressses and Keys](./protocol/addresses_keys.md) section describes the Penumbra key hierarchy and diversified addresses.

- The [Notes](./protocol/notes.md) section describes Penumbra's private notes and their contents.

- The [Transaction Cryptography](./protocol/transaction_crypto.md) section describes the symmetric keys used at the level of an individual transaction.

## Notation

We use the following notation in this chapter:

* $\mathbb G$ denotes the `decaf377` group;
* $\mathbb F_q$ denotes the BLS12-377 scalar field of order $q$;
* $\mathbb F_r$ denotes the `decaf377` scalar field of order $r$.
