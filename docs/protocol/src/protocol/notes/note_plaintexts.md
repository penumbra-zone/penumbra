# Note Plaintexts

Plaintext notes contain:

* the value to be transmitted which consists of an integer amount $v$ along with a scalar (32 bytes) $ID$ identifying the asset.
* $rseed$, a 32-byte random value, which will later be used to derive the note blinding factor used for the
note commitment and an ephemeral secret key.
* the destination address, described in more detail in the [Addressses](../addresses_keys/addresses.md) section.

The note can only be spent by the holder of the spend key that corresponds to the diversified transmission key $pk_d$ in the note. 
