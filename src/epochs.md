# Epochs

Penumbra organizes blocks into epochs.  The number of blocks in each epoch is chosen relative to the block interval so that epochs last approximately 24 hours.  For instance, an 8-second block interval would imply $86400/8 = 10800$ blocks per epoch.

Most state changes (e.g., transfers from one address to another) are applied at block boundaries.  However, state changes related to delegation and consensus are generally applied only at epoch boundaries.

The validator set and its consensus weighting is determined on a per-epoch basis, and only changes at epoch boundaries, except in the case of slashing, which causes a validator to be removed from the validator set immediately.
