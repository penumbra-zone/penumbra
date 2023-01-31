# Transaction Cryptography

This section describes the transaction-level cryptography that is used in Penumbra
actions to symmetrically encrypt and decrypt swaps, notes, and memos.

For the symmetric encryption described in this section, we use ChaCha20-Poly1305 ([RFC-8439]).
It is security-critical that `(key, nonce)` pairs are not reused. We do this by either:

* deriving per-action keys from ephemeral shared secrets using a fixed nonce (used for notes and memo keys),
* deriving per-action keys from the outgoing viewing key using a nonce derived from a commitment.

We use each key at maximum once with a given nonce. We describe the nonces and keys below.

## Nonces

We use a different nonce to encrypt each type of item using the following `[u8; 12]` arrays:

* Notes: `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
* Memos: `[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
* Swaps: we use the first 12 bytes of the swap commitment
* Memo keys: `[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`

## Keys

We have several keys involved in transaction-level symmetric cryptography:
* A *shared secret* derived between sender and recipient,
* A *per-action payload key* used to encrypt a single note, or memo key (one of each type).
It is derived from the shared secret. It is used for a single action.
* A *per-action swap payload key* used to encrypt a single swap. It is derived from the outgoing viewing key. It is used for a single action.
* A *random payload key* used to encrypt a memo. It is generated randomly and is shared between all
actions in a given transaction.
* An *outgoing cipher key* used to encrypt key material (the shared secret) to enable the sender to 
later decrypt an outgoing swap, note, or memo.
* An *OVK wrapped key* - this is the shared secret encrypted using the sender's outgoing
cipher key. A sender can later use this field to recover the shared secret
they used when encrypting the note, swap, or memo to a recipient.
* A *wrapped memo key* - this is the memo key encrypted using the per-action payload key. The memo
key used for a (per-transaction) memo is shared between all outputs in that transaction. The wrapped
key is encrypted to their per-action payload key.

In the rest of this section, we describe how each key is derived.

### Shared Secret

A shared secret `shared_secret` is derived between sender and recipient by performing Diffie-Hellman
key exchange between:
* an ephemeral secret ($esk \in \mathbb F_r$) and the diversified transmission key $pk_d$ of the recipient (described in more detail in the [Addressses](../addresses_keys/addresses.md) section), 
* the ephemeral public key $epk$ (where $epk = [esk] B_d$), provided as a public field in the action, and the recipient's incoming viewing key $ivk$.

This allows both sender and recipient to generate the shared secret based on the keys they posess.

### Per-action Payload Key: Notes and Memo Keys

The symmetric per-action payload key is a 32-byte key derived from the `shared_secret`, the $epk$ and
personalization string "Penumbra_Payload":

```
action_payload_key = BLAKE2b-512("Penumbra_Payload", shared_secret, epk)
```

This symmetric key is then used with the nonces specified above to encrypt a memo key or note.
It should not be used to encrypt two items of the same type.

### Per-action Payload Key: Swaps

The symmetric per-action payload key is a 32-byte key derived from the outgoing viewing
key `ovk`, the swap commitment `cm`, and personalization string "Penumbra_Payswap":

```
action_payload_key = BLAKE2b-512("Penumbra_Payswap", ovk, cm)
```

This symmetric key is used with the nonces specified above to encrypt a swap only.
We use the outgoing viewing key for swaps since third parties cannot create
swaps for a given recipient, i.e. the user only sends swaps to themselves.

### Random Payload Key: Memos

The random payload key is a 32-byte key generated randomly. This
symmetric key is used with the nonces specified above to encrypt memos only.
This key is provided to all output actions in a given transaction, as all outputs
should be able to decrypt the per-transaction memo.

### Outgoing Cipher Key

The symmetric outgoing cipher key is a 32-byte key derived from the sender's outgoing viewing key
$ovk$, the value commitment $cv$, the note commitment $cm$, the ephemeral 
public key $epk$, and personalization string "Penumbra_OutCiph":

```
outgoing_cipher_key = BLAKE2b-512("Penumbra_OutCiph", ovk, cv, cm, epk)
```

All inputs except the outgoing viewing key are public. The intention of the
outgoing cipher key is to enable the sender to view their outgoing transactions
using only their outgoing viewing key. The outgoing cipher key is used to encrypt data that
they will need to later reconstruct any outgoing transaction details.

### OVK Wrapped Key

To decrypt outgoing notes or memo keys, the sender needs to store the shared secret encrypted
using the outgoing cipher key described above. This encrypted data,
48-bytes in length (32 bytes plus 16 bytes for
authentication) we call the OVK wrapped key and is saved on `OutputBody`.

The sender later scanning the chain can:

1. Derive the outgoing cipher key as described above using their outgoing viewing
key and public fields.
2. Decrypt the OVK wrapped key.
3. Derive the per-action payload key and use it to decrypt the corresponding memo or note.

### Wrapped Memo Key

The wrapped memo key is 48 bytes in length (32 bytes plus 16 bytes for authentication).
It is saved on the `OutputBody` and is encrypted using the per-action payload key.

[RFC-8439]: https://datatracker.ietf.org/doc/rfc8439/
