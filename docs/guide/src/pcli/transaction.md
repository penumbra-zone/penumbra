## Sending Transactions

Now, for the fun part: sending transactions. If you have someone else's testnet address, you can
send them any amount of any asset you have.

First, use balance to find the amount of assets you have:

```bash
cargo run --release --bin pcli view balance
```

Second, if I wanted to send 10 penumbra tokens
to my friend, I could do that like this (filling in their full address at the end):

```bash
cargo run --quiet --release --bin pcli tx send 10penumbra --to penumbrav2t...
```

Notice that asset amounts are typed amounts, specified without a space between the amount (`10`)
and the asset name (`penumbra`). If you have the asset in your wallet to send, then so it shall be done!

## Staking

In addition, to sending an asset, one may also stake penumbra tokens to validators.

Find a validator to stake to:

```bash
cargo run --release --bin pcli query validator list
```

Copy and paste the identity key of one of the validators to stake to, then construct the staking tx:

```bash
cargo run --release --bin pcli tx delegate 10penumbra --to penumbravalid...
```

To undelegate from a validator, use the `pcli tx undelegate` command, passing it the typed amount of
delegation tokens you wish to undelegate. Wait a moment for the network to process the undelegation,
then reclaim your funds:

```bash
cargo run --release --bin pcli tx undelegate-claim
```

Inspect the output; a message may instruct you to wait longer, for a new epoch. Check back and rerun the command
later to add the previously delegated funds to your wallet.

## Governance

Penumbra features on-chain governance similar to Cosmos Hub where anyone can submit proposals and
both validators and delegators to vote on them. Penumbra's governance model incorporates a single
DAO account, into which anyone can freely deposit, but from which only a successful governance vote
can spend. For details on using governance, see the [governance section](./governance.md).

## Swapping Assets

One of the most exciting features of Penumbra is that by using IBC (inter-blockchain communication)
and our shielded pool design, **any** tokens can be exchanged in a private way.

**NOTE:** Since there's not yet a way to open liquidity positions, we have implemented a stub
[Uniswap-V2-style](https://uniswap.org/blog/uniswap-v2) constant-product market maker with some
hardcoded liquidity for a few trading pairs: `gm:gn`, `penumbra:gm`, and `penumbra:gn`.

This allows testing the shielded pool integration, and will cause a floating exchange rate based
on the volume of swaps occurring in each pair.

You can check the current reserves for a trading pair using the `cpmm-reserves` dex query:

```bash
cargo run --release --bin pcli -- q dex cpmm-reserves gm:penumbra
```

If you wanted to exchange 1 `penumbra` tokens for `gm` tokens, you could do so like so:

```bash
cargo run --release --bin pcli -- tx swap --into gm 1penumbra
```

This will handle generating the swap transaction and you'd soon have the market-rate equivalent of 1 `penumbra`
in `gm` tokens returned to you, or the original investment of 1 `penumbra` tokens returned if there wasn't
enough liquidity available to perform the swap.
