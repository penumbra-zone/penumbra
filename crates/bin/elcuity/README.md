# elcuity

This is an experimental tool to add in generating activity for testing the Liquidity Tournament.

The command has several commands which are intended to run in the background, and generate activity.
This activity takes the form of either
- voting in the tournament,
- providing liquidity,
- or swapping between assets in a loop.

All commands start as:
```bash
elcuity --grpc-url <...> --view-service <...>
```
The GRPC url is intended to be that of a remote node.
The view service is intended to be a URL to a view service speaking GRPC.
This view service must also support custody, to allow for signing transactions.

To get such a view service, you can use `pclientd`.

When initializing `pclientd` for the first time, you'll want to use the `--custody` flag,
so that pclientd has a spend key it can use for signing transactions.

## Voting

```bash
elcuity vote --for DENOM
```

This will repeatedly vote in the liquidity tournament, for a given denom.
This must be the base denom of an IBC transfer asset.

This command will not delegate for you, so you must have staked UM prior
to running this command to use for voting.

## Liquidity Provision

```bash
elcuity lp --for DENOM --liquidity-target AMOUNT --default-price PRICE 
```

This command will provide liquidity between UM and DENOM.
The strategy here is to quote a given price with a tighter spread than what's observed
on the market.
The liquidity target is in terms of UM, and will be divided equally between the two assets.
In more detail, half the target will be the reserves in UM, and the other half will be assigned
to the other asset, after doing a price conversion based on the position's price.
e.g. if you have a target of 1000, and are quoting a price of 2 UM / DENOM, then you will provide
500 UM and 250 DENOM in reserves, because each DENOM is worth 2 UM.

The command will also only provide balances that it actually has, so if it only has, say, 200 DENOM
(continuing this example), it will limit the reserves to just that.
Furthermore, to guarantee the ability to pay fees, it will never allocate more than
90% of its UM balance to liquidity provision.
So, if you have 500 UM in balance, you will only provide 450 UM in liquidity (in this example).

The default price is optional, and is used to provide a quote price in case no market price exists.

## Swap Cycling

```bash
elcuity swap --starting-amount AMOUNT --cyle ASSET1 --cycle ASSET2 ...
```

This will repeatedly swap assets in a cycle:

```txt
UM -> ASSET1 -> ASSET2 -> ... -> UM -> ASSET1 -> ...
```

The starting amount is in UM, and then further swaps will consume whatever value was produced by
the previous swap.
Once the cycle repeats, and you arrive back at UM, the value is also topped up to reach
the starting amount, if necessary.
This prevents the cycle from ending prematurely because of value loss through fees.
Because of this, this command can actually drain your account of funds, through charity
to liquidity providers!

