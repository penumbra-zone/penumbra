# Transaction Cryptography

This section describes the transaction-level cryptography that is used in Penumbra
actions to symmetrically encrypt and decrypt swaps, notes, and memos.

For our symmetric encryption, we use ChaCha20-Poly1305 ([RFC-8439]).
It is security-critical that `(key, nonce)` pairs are not reused. We do this by
deriving keys from ephemeral shared secrets, and using each key at maximum once
with a given nonce. We describe the nonces and keys below.

## Nonces

We use a different nonce to encrypt each type of item using the following `[u8; 12]` arrays:

* Notes: `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
* Memos: `[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
* Swaps: `[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`

This enables the same key to be used in the
case of e.g. a swap with an associated note, or a note with an associated memo.

## Keys

We have four primary keys involved in transaction-level symmetric cryptography:
* A *shared secret* derived between sender and recipient,
* A *payload key* used to encrypt a single swap, note, or memo (one of each type). 
It is derived from the shared secret.
* An *outgoing cipher key* used to encrypt key material (the shared secret) to enable the sender to 
later decrypt an outgoing swap, note, or memo.
* An *OVK wrapped key* - this is the shared secret encrypted using the sender's outgoing
cipher key. A sender can later use this field to recover the shared secret
they used when encrypting the note, swap, or memo to a recipient.

In the rest of this section, we describe how each key is derived.

### Shared Secret

A shared secret `shared_secret` is derived between sender and recipient by performing Diffie-Hellman
key exchange between:
* an ephemeral secret ($esk \in \mathbb F_r$) and the diversified transmission key $pk_d$ of the recipient (described in more detail in the [Addressses](../addresses_keys/addresses.md) section), 
* the ephemeral public key $epk$ (where $epk = [esk] B_d$), provided as a public field in the action, and the recipient's incoming viewing key $ivk$.

This allows both sender and recipient to generate the shared secret based on the keys they posess.

### Payload Key

The symmetric payload key is a 32-byte key derived from the `shared_secret` and the $epk$:

```
payload_key = BLAKE2b-512(shared_secret, epk)
```

This symmetric key is then used with the nonces specified above to encrypt a memo,
note, or swap. It should not be used to encrypt two items of the same type.

### Outgoing Cipher Key

The symmetric outgoing cipher key is a 32-byte key derived from the sender's outgoing viewing key
$ovk$, the value commitment $cv$, the note commitment $cm$, and the ephemeral 
public key $epk$:

```
outgoing_cipher_key = BLAKE2b-512(ovk, cv, cm, epk)
```

All inputs except the outgoing viewing key are public. The intention of the
outgoing cipher key is to enable the sender to view their outgoing transactions
using only their outgoing viewing key. The outgoing cipher key is used to encrypt data that
they will need to later reconstruct any outgoing transaction details.

### OVK Wrapped Key

To decrypt outgoing notes, memos, or swaps, the sender needs to store the shared secret encrypted
using the outgoing cipher key described above. This encrypted data,
48-bytes in length (32 bytes plus 16 bytes for
authentication) we call the OVK wrapped key and is saved on `OutputBody`.

The sender later scanning the chain can:

1. Derive the outgoing cipher key as described above using their outgoing viewing
key and public fields.
2. Decrypt the OVK wrapped key.
3. Derive the payload key and use it to decrypt the corresponding memo, note, or swap.

[RFC-8439]: https://datatracker.ietf.org/doc/rfc8439/
