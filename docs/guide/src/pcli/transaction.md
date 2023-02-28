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

Penumbra features on-chain governance in the style of Cosmos Hub. Anyone can submit a new governance
proposal for voting by escrowing a _proposal deposit_, which will be held until the end of the
proposal's voting period. If the proposal is not slashed by voters, the deposit will then be
returned; if it is slashed, then the deposit is burned.

### Submitting A Proposal

To submit a proposal, first generate a proposal template for the kind of proposal you want to
submit. For example, suppose we want to create a signaling proposal:

```bash
cargo run --release --bin pcli tx proposal template --kind signaling --file proposal.json
```

This outputs a JSON template for the proposal to the file `proposal.json`, where you can edit the
details to match what you'd like to submit.

Once you're ready to submit the proposal, you can submit it. Note that you do not have to explicitly
specify the proposal deposit in this action; it is determined automatically based on the chain
parameters.

```bash
cargo run --release --bin pcli tx proposal submit --file proposal.json
```

The proposal deposit will be immediately escrowed and the proposal voting period will start in the
very next block. When voting finishes, if your proposal has not been slashed, it will automatically
be refunded to you with no action required on your part.

### Getting Proposal Information

To see information about the currently active proposals, including your own, use the `pcli query
proposal` subcommand.

To list all the active proposals by their ID, use:

```bash
cargo run --release --bin pcli query governance list-proposals
```

Other proposal query commands all follow the form:

```bash
cargo run --release --bin pcli query governance proposal [PROPOSAL_ID] [QUERY]
```

These are the queries currently defined:

- `definition` gets the details of a proposal, as the submitted JSON;
- `state` gets information about the current state of a proposal (voting, withdrawn, or finished,
  along with the reason for withdrawal if any, and the outcome of finished proposals);
- `period` gets the voting start and end block heights of a proposal;
- `tally` gets the current tally of a proposal's votes;
- `validator-votes` gets the list of public validator votes on the proposal, by identity key

### Withdrawing A Proposal

If you want to withdraw a proposal that you have made (perhaps because a better proposal has come to
community consensus), you can do so before voting concludes. Note that this does not make you immune
to losing your deposit by slashing, as withdrawn proposals can still be voted on and slashed.

```bash
cargo run --release --bin pcli tx proposal withdraw 0 --reason "some human-readable reason for withdrawal"
```

### Voting On A Proposal

Delegators cannot yet vote on proposals (this functionality is coming soon!). If you are a
validator, you can vote on a proposal using the `validator vote` subcommand of `pcli`. For example,
if you wanted to vote "yes" on proposal 1, you would do:

```bash
cargo run --release --bin pcli validator vote yes --on 1
```

Validators, like delegators, cannot change their votes after they have voted.

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
