# Note Ciphertexts

Encrypting a note plaintext involves the following steps:

1. Derive Ephemeral Secret Key: Use [decaf377-ka](https://github.com/penumbra-zone/decaf377-ka) to derive an ephemeral secret key *esk*.

2. Derive Diversified Public Key: Generate a diversified public key from the secret key *epk*.

3. Shared Secret Derivation: Perform a secure Diffie-Hellman key exchange to derive the shared secret between the sender and recipient.

4. Symmetric Key Generation: Generate a symmetric [ChaCha20-Poly1305](https://protocol.penumbra.zone/main/addresses_keys/transaction_crypto.html#random-memo-key) payload key from the shared secret and ephemeral public key.

5. Encryption: Encrypt with note plaintext, represented as a vector of bytes, using the ChaCha20-Poly1305 encryption algorithm.

6. Construct the encrypted note ciphertext object.

The note ciphertext is **176** bytes in length.
