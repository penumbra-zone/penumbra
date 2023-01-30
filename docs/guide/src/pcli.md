# Using pcli

This section describes how to use `pcli`, the command line client for Penumbra:

- [Installation](./pcli/install.md) describes how to compile and run `pcli`;
- [Generating a Wallet](./pcli/wallet.md) describes how to generate a wallet and use the testnet faucet;
- [Updating pcli](./pcli/update.md) describes how to update to a newly released testnet from a previous testnet;
- [Viewing Balances](./pcli/balance.md) describes how to view balances;
- [Sending Transactions](./pcli/transaction.md) describes how to send funds.

Penumbra is a private blockchain, so the public chain state does not reveal any
private user data.  By default, `pcli` includes a _view service_ that
synchronizes with the chain and scans with a viewing key.  However, it's also
possible to run the view service as a standalone `pclientd` daemon:

- [Using `pcli` with `pclientd`](./pcli/pclientd.md) describes how to use `pcli` with `pclientd`.

### Please submit any feedback and bug reports

Thank you for helping us test the Penumbra network! If you have any feedback, please let us know in
the `#testnet-feedback` channel on our [Discord](https://discord.gg/hKvkrqa3zC). We would love to know about bugs, crashes,
confusing error messages, or any of the many other things that inevitably won't quite work yet. Have
fun! :)

### Diagnostics and Warnings

By default, `pcli` prints a warning message to the terminal, to be sure that people understand that this is *unstable, unfinished, pre-release software*.
To disable this warning, export the `PCLI_UNLEASH_DANGER` environment variable.
