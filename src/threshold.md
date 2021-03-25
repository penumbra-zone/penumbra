# Homomorphic Threshold Decryption

Penumbra requires a homomorphic encryption scheme operating on `i64` values that supports threshold decryption and distributed key generation.
This scheme is used to allow transactions to include encrypted values that can be aggregated over an entire block or epoch, with the validators revealing only the aggregate value, not the value from any individual transaction.

This encryption scheme only needs to work on `i64` values, not arbitrary data, such as an entire transaction.  Penumbra does not use threshold decryption to unseal entire encrypted transactions, because Penumbra transactions are constructed not to reveal any unnecessary information.

At the beginning of each epoch, the validator set performs distributed key generation for a homomorphic encryption scheme to produce a decryption key collectively controlled by the validators (on an equal basis, not a stake-weighted basis) and includes the encryption key in the first block of the epoch.

Because this key is only available after the first block of each epoch, some transactions cannot occur in the first block itself.  Assuming a block interval similar to the Cosmos Hub, this implies an ~8-second processing delay once per day, a reasonable tradeoff against the complexity of phased setup procedures.

A sketch of one construction is as follows.

Write a value $v \in [-2^{63}, 2^{63}) $ in radix $11$ with signed coefficients, i.e., as $$v = v_0 + v_1 2^{11} + \cdots + v_5 2^{55}$$ with $v_i \in [-2^{11}, 2^{11})$.  Encode each coefficient $v_i$ as $v_i B$ and use ElGamal encryption to form the ciphertext
$$
\operatorname{Enc}_D(v) = (\operatorname{Enc}_D(v_0 B), \ldots, \operatorname{Enc}_D(v_5 B)).
$$
Each ElGamal ciphertext consists of two group elements; if group elements can be encoded in 32 bytes, this gives a 384-byte ciphertext.  To decrypt $\operatorname{Enc}_D(v)$, use ElGamal decryption to obtain the group elements $(v_0 B, \ldots, v_5 B)$, and then use a lookup table to recover $v_i$ from $v_i B$, or fail if the value is unknown.

This can in principle be done inside of a zkSNARK circuit if the underlying group is an embedded elliptic curve, together with certification that the ciphertext was correctly formed with in-range coefficients.

Addition and subtraction of ciphertexts are done componentwise, using the homomorphic property of ElGamal encryptions, and the fact that $v_i B + w_i B = (v_i + w_i)B$.

Adding $n = 2^k$ ciphertexts of values whose coefficients were bounded as $|v_{i,k}| \leq 2^{11}$ results in a sum whose coefficients $w_i$ are bounded as $|w_i| \leq n 2^{11}  = 2^{11 + k}$.  Choosing $k = 7$ and accounting for sign means that a lookup table of size $2\cdot 2^{11 + 7} = 2^{19}$ is sufficient to decrypt sums of up to 128 well-formed ciphertexts. Sums of more than 128 ciphertexts can be handled by summing blocks of 128, decrypting the block sum, and summing the plaintexts.

Unfortunately, this method reveals slightly more information about a sum of encrypted summands than would be ideal.  Ideally, it would reveal only the sum of the encrypted summands, but in fact it reveals the sum of each radix-$11$ chunk, without carrying between them.  Carrying [collapses information about the summands](https://www.jstor.org/stable/3072368), but that information is revealed by this scheme.  This seems unlikely to be a problem in practice, but it is worth quantifying.

## TODO

- [ ] the bounds above are a ballpark estimation; refine them and make them precise
- [ ] work out integration with ABCI++ protocol phases
