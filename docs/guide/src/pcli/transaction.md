## Sending Transactions

Now, for the fun part: sending transactions. If you have someone else's testnet address, you can
send them any amount of any asset you have.

First, use balance to find the amount of assets you have:

```bash
pcli view balance
```

Second, if I wanted to send 10 penumbra tokens
to my friend, I could do that like this (filling in their full address at the end):

```bash
pcli tx send 10penumbra --to penumbrav2t...
```

Notice that asset amounts are typed amounts, specified without a space between the amount (`10`)
and the asset name (`penumbra`). If you have the asset in your wallet to send, then so it shall be done!

## Staking

In addition, to sending an asset, one may also stake penumbra tokens to validators.

Find a validator to stake to:

```bash
pcli query validator list
```

Copy and paste the identity key of one of the validators to stake to, then construct the staking tx:

```bash
pcli tx delegate 10penumbra --to penumbravalid...
```

To undelegate from a validator, use the `pcli tx undelegate` command, passing it the typed amount of
delegation tokens you wish to undelegate. Wait a moment for the network to process the undelegation,
then reclaim your funds:

```bash
pcli tx undelegate-claim
```

Inspect the output; a message may instruct you to wait longer, for a new epoch. Check back and rerun the command
later to add the previously delegated funds to your wallet.

## Governance

Penumbra features on-chain governance similar to Cosmos Hub where anyone can submit proposals and
both validators and delegators to vote on them. Penumbra's governance model incorporates a single
Community Pool account, into which anyone can freely deposit, but from which only a successful governance vote
can spend. For details on using governance, see the [governance section](./governance.md).

## Managing Liquidity Positions

Penumbra's decentralized exchange ("dex") implementation allows users to create their own on-chain
liquidity positions. The basic structure of a liquidity position expresses a relationship between two
assets, i.e. "I am willing to buy `100penumbra` at a price of `1gm` each, with a fee of `20bps` (base points)" or
"I am willing to sell `100penumbra` at a price of `1gm` each, with a fee of `10bps`".

### Opening a Liquidity Position

The basic commands for opening liquidity positions are `tx position order buy` and `tx position order sell`.

To open an order buying `10cube` at a price of `1penumbra` each, with no fee, you'd do the following:

```bash
pcli tx position order buy 10cube@1penumbra
```

Similarly, to open an order selling `100penumbra` at a price of `5gm` each, with a `20bps` fee on transactions
against the liquidity position, you would append `/20bps` at the end of the order, like as follow:

```bash
pcli tx position order sell 100penumbra@5gm/20bps
```

After opening the position, you'll see that your account has been deposited an "LPNFT" representing the open position:

```bash
$ pcli view balance

 Account  Amount
 0        1lpnft_opened_plpid1hzrzr2myjw508nf0hyzehl0w0x2xzr4t8vwe6t3qtnfhsqzf5lzsufscqr
```

### Closing a Liquidity Position

If you have an open liquidity position, you may close it, preventing further trading against it.

```bash
pcli tx position close plpid1hzrzr2myjw508nf0hyzehl0w0x2xzr4t8vwe6t3qtnfhsqzf5lzsufscqr
```

This will subtract the opened LPNFT and deposit a closed LPNFT into your balance:

```bash
$ pcli view balance

 Account  Amount
 0        1lpnft_closed_plpid1hzrzr2myjw508nf0hyzehl0w0x2xzr4t8vwe6t3qtnfhsqzf5lzsufscqr
```

You also have the option to close **all** liquidity positions associated with an address at once. This is useful if you have many individual positions, e.g. due to
trading function approximation:

```bash
pcli tx position close-all
```

### Withdrawing a Liquidity Position

If you have a closed liquidity position, you may withdraw it, depositing the reserves in the trading position into your balance.

```bash
pcli tx position withdraw plpid1hzrzr2myjw508nf0hyzehl0w0x2xzr4t8vwe6t3qtnfhsqzf5lzsufscqr
```

This will subtract the closed LPNFT and deposit a withdrawn LPNFT into your balance, along with any reserves belonging to the trading position:

```bash
$ pcli view balance

 Account  Amount
 0        1lpnft_withdrawn_plpid1hzrzr2myjw508nf0hyzehl0w0x2xzr4t8vwe6t3qtnfhsqzf5lzsufscqr
 0        1cube
```

You also have the option to withdraw **all** liquidity positions associated with an address at once. This is useful if you have many individual positions, e.g. due to
trading function approximation:

```bash
pcli tx position withdraw-all
```

## Swapping Assets

One of the most exciting features of Penumbra is that by using IBC (inter-blockchain communication)
and our shielded pool design, **any** tokens can be exchanged in a private way.

Swaps take place against the on-chain liquidity positions described earlier in the guide.

If you wanted to exchange 1 `penumbra` tokens for `gm` tokens, you could do so like so:

```bash
pcli tx swap --into gm 1penumbra
```

This will handle generating the swap transaction and you'd soon have the market-rate equivalent of 1 `penumbra`
in `gm` tokens returned to you, or the original investment of 1 `penumbra` tokens returned if there wasn't
enough liquidity available to perform the swap.

## Replicating a UniswapV2 (`x*y=k`) pool

Penumbra's constant-price pool is a versatile market primitive, allowing users extensive control over their trading strategies. It's not solely for active DEX quoters; with our AMM replication tool, users can emulate any passive AMM of their choice. The testnet comes with a built-in UniswapV2 replicator that is utilized as such:

```bash
pcli tx lp replicate xyk <TRADING_PAIR> <QUANTITY> [--current-price AMT] [--fee-bps AMT]
```

For instance, to provide ~100penumbra and ~100test_usd liquidity on the `penumbra:test_usd` pair with a pool fee of `33bps`, run:

```bash
pcli tx lp replicate xyk penumbra:test_usd 100penumbra --fee-bps 33
```

You will be prompted a disclaimer which you should read carefully, and accept or reject by pressing "y" for yes, or "n" for no.

The replicating market makers tool will then generate a list of positions that you can submit by pressing "y", or reject by pressing "n".

There are other pairs available that you can try this tool on, for example `gm:gn` or `gm:penumbra`.

## IBC withdrawals

<!--
N.B. These steps require a running Hermes deployment, specifically one that
has been configured between the Osmosis testnet and the *current* Penumbra testnet.
-->

Penumbra aims to implement full IBC support for cross-chain asset transfers. For now, however,
we're only running a relayer between the Penumbra testnet and the [Osmosis testnet] chains.
For Testnet 64 Titan, the channel information is:

<!--
To update the information below, update the Hermes config, then run:
`pcli q ibc channels | jq '.[0]'` and confirm that output matches what Hermes emitted
during setup.
-->

```
{
  "state": 3,
  "ordering": 1,
  "counterparty": {
    "port_id": "transfer",
    "channel_id": "channel-4687"
  },
  "connection_hops": [
    "connection-1"
  ],
  "version": "ics20-1",
  "port_id": "transfer",
  "channel_id": "channel-1"
}
```

The output above shows that the IBC channel id on Penumbra is 0, and on Osmosis it's 4544.
There's one more piece of information we need to make an IBC withdrawal: the appropriate IBC
timeout height, which is composed of two values: `<counterparty_chain_id_revision>-<counterparty_chain_block_height>`.
For the Osmosis testnet, as of 2023Q4, the chain id is `osmo-test-5`, meaning the chain id revision is `5`.
So a value like `5-5000000` (i.e. revision 5 at height 5 million) will work.

To initiate an IBC withdrawal from Penumbra testnet to Osmosis testnet:

```bash
pcli tx withdraw --to <OSMOSIS_ADDRESS> --channel <CHANNEL_ID> 5gm --timeout-height 5-5000000
```

Unfortunately the CLI tooling for Osmosis is cumbersome. For now, use `rly` as a user agent
for the Osmosis testnet, as described in the [IBC dev docs](../dev/ibc.md).

[Osmosis testnet]: https://docs.osmosis.zone/networks/join-testnet/
