# Using pcli

This section describes how to use `pcli`, the command line client for Penumbra:

- [Installation](./pcli/install.md) describes how to compile and run `pcli`;
- [Generating a Wallet](./pcli/wallet.md) describes how to generate a wallet and use the testnet faucet;
- [Viewing Balances](./pcli/balance.md) describes how to view balances;
- [Sending Transactions](./pcli/send.md) describes how to send funds.

### Please submit any feedback and bug reports

Thank you for helping us test the Penumbra network! If you have any feedback, please let us know in
the `#testnet-feedback` channel on our [Discord]. We would love to know about bugs, crashes,
confusing error messages, or any of the many other things that inevitably won't quite work yet. Have
fun! :)


### Diagnostics and Warnings

By default, `pcli` prints a warning message to the terminal, to be sure that people understand that this is *unstable, unfinished, pre-release software*.
To disable this warning, export the `PCLI_UNLEASH_DANGER` environment variable.

When working with `pcli`, the level of diagnostic information printed is
dependent on the `RUST_LOG` environment variable. To see progress updates and
other logged information while running `pcli`, we recommend you set `export
RUST_LOG=info` in your terminal.
