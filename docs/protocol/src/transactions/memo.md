# Transaction Memo

The transaction-level memo field is optional, and will be present _if and only if_ the transaction has outputs. A consensus rule will reject transactions with memos that have no outputs, and transactions that have outputs but lack a memo.

## Memo Plaintext

The plaintext of the memo contains:

* a return address (80 bytes for Penumbra addresses)
* a text string that is 432 bytes in length

## Privacy

The transaction-level encrypted memo is visible only to the sender and receiver(s) of the transaction.

Each memo is encrypted using the *Memo Key*, a symmetric ChaCha20-Poly1305 key generated randomly as described [here](../addresses_keys/transaction_crypto.md#random-memo-key). The Memo Key is then encrypted using data from each output following [this section](../addresses_keys/transaction_crypto.md#per-action-payload-key-notes-and-memo-key). This *Wrapped Memo Key* is then added to each individual [Output](../shielded_pool/action/output.md#output-body).
