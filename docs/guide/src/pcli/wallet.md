# Generating a Wallet

On first [installation of `pcli`](./install.md), you will need to generate a fresh wallet to use with Penumbra.

The `pcli init` command will generate a configuration file, depending on the
custody backend used to store keys.

There are currently three custody backends:

1. The [`softkms` backend](./wallet/softkms.md) is a good default choice for low-security use cases.  It stores keys unencrypted in a local config file.
2. The [threshold backend](./wallet/threshold.md) is a good choice for high-security use cases. It provides a shielded multisig, with key material sharded over multiple computers.
3. The `view-only` backend has no custody at all and only has access to viewing keys.

After running `pcli init` with one of the backends described above, `pcli` will be initialized.

Penumbra's design automatically creates `2^32` (four billion) numbered accounts
controlled by your wallet.

To generate the address for a numbered account, use `pcli view address`:
```bash
$ pcli view address 0
penumbrav2t1...
```
You can also run `pcli view address` on an address to see which account it corresponds to:
```bash
$ pcli view address penumbra1...
Address is viewable with this full viewing key. Account index is 0.
```

Addresses are opaque and do not reveal account information. Only you, or someone
who has your viewing key, can decrypt the account information from the address.

### Getting testnet tokens on Discord in the `#testnet-faucet` channel

In order to use the testnet, it's first necessary for you to get some testnet tokens. The current
way to do this is to join our [Discord](https://discord.gg/hKvkrqa3zC) and post your address in the `#testnet-faucet` channel.
We'll send your address some tokens on the testnet for you to send to your friends! :)

Just keep in mind: **testnet tokens do not have monetary value**, and in order to keep the
signal-to-noise ratio high on the server, requests for tokens in other channels will be deleted
without response. Please do not DM Penumbra Labs employees asking for testnet tokens; the correct
venue is the dedicated channel.
