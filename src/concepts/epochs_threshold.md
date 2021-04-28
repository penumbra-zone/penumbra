# Epochs and Threshold Decryption

Penumbra organizes blocks into epochs.  The number of blocks in each epoch is
chosen relative to the block interval so that epochs last approximately 24
hours.  For instance, an 8-second block interval would imply $86400/8 =
10800$ blocks per epoch.

Most state changes (e.g., transfers from one address to another) are applied
at block boundaries.  However, state changes related to delegation and
consensus are generally applied only at epoch boundaries.

The validator set and its consensus weighting is determined on a per-epoch
basis, and only changes at epoch boundaries, except in the case of slashing,
which causes a validator to be removed from the validator set immediately.

Penumbra requires a homomorphic encryption scheme operating on `i64` values
that supports threshold decryption and distributed key generation.  This
scheme is used to allow transactions to include encrypted values that can be
aggregated and then decrypted, with the validators revealing only
the aggregate value, not the value from any individual transaction.

The choice to restrict validator changes to epoch boundaries plays a key role
in Penumbra's design, in two ways:

1.  it allows the validators to share control of a single threshold key over
the entire epoch, allowing aggregation of values in different blocks;

2.  it provides a time interval over which changes to validators' delegations
can be aggregated, enhancing delegator privacy.

## Homomorphic Threshold Decryption

This encryption scheme only needs to work on `i64` values, not arbitrary
data, such as an entire transaction.  Penumbra does not use threshold
decryption to unseal entire encrypted transactions, because Penumbra
transactions are constructed not to reveal any unnecessary information.

At the beginning of each epoch, the validator set performs distributed key
generation for a homomorphic encryption scheme to produce a decryption key
collectively controlled by the validators (on an equal basis, not a
stake-weighted basis) and includes the encryption key in the first block of
the epoch.

Because this key is only available after the first block of each epoch, some
transactions cannot occur in the first block itself.  Assuming a block
interval similar to the Cosmos Hub, this implies an ~8-second processing
delay once per day, a reasonable tradeoff against the complexity of phased
setup procedures.
