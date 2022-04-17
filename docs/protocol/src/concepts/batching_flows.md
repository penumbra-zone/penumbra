# Batching Flows

Penumbra's ledger records value as it moves between different economic roles –
for instance, movement between unbonded stake and delegation tokens, movement
between different assets as they are traded, etc.   This creates a tension
between the need to reveal the total amount of value in each role as part of the
public chain state, and the desire to shield value amounts in individual
transactions.

To address this tension, Penumbra provides a mechanism to aggregate value flows
across a batch of transactions, revealing the only the total amount and not the
value contributed by each individual transaction.  This mechanism is built using
an integer-valued homomorphic encryption scheme that supports threshold
decryption, so that the network's validators can jointly control a decryption
key.

Transactions that contribute value to a batch contain an encryption of the
amount.  To flush the batch, the validators sum the ciphertexts from all
relevant transactions to compute an encryption of the batch total, then jointly
decrypt it and commit it to the chain.

This mechanism doesn't require any coordination between the users whose
transactions are batched, but it does require that the validators create and
publish a threshold decryption key.  To allow batching across block boundaries,
Penumbra organizes blocks into epochs, and applies changes to the validator set
only at epoch boundaries.  Decryption keys live for the duration of the epoch,
allowing value flows to be batched over any time interval from 1 block up to the
length of an epoch. We propose epoch boundaries on the order of 1-3 days.

At the beginning of each epoch, the validator set performs distributed key
generation for to produce a decryption key jointly controlled by the
validators (on an approximately stake-weighted basis) and includes the
encryption key in the first block of the epoch.

Because this key is only available after the first block of each epoch, some
transactions cannot occur in the first block itself.  Assuming a block
interval similar to the Cosmos Hub, this implies an ~8-second processing
delay once per day, a reasonable tradeoff against the complexity of phased
setup procedures.
