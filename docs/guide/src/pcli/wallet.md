# Generating a Wallet

On first installation of `pcli`, you will need to generate a fresh wallet to use with Penumbra. You
should see something like this:

```bash
\$ pcli keys generate


YOUR PRIVATE SEED PHRASE: [...]
DO NOT SHARE WITH ANYONE!
Saving backup wallet to /home/\$USER/.local/share/penumbra-testnet-archive/.../custody.json
```

Penumbra's design automatically creates many (`u64::MAX`) publicly unlinkable addresses which all
correspond to your own wallet. When you first created your wallet above, `pcli` initialized all
of your wallet addresses, which you can view like this:

```bash
\$ pcli view address 0
penumbrav2t1...
```

### Getting testnet tokens on the [Discord] in the `#testnet-faucet` channel

In order to use the testnet, it's first necessary for you to get some testnet tokens. The current
way to do this is to join our [Discord](https://discord.gg/hKvkrqa3zC) and post your address in the `#testnet-faucet` channel.
We'll send your address some tokens on the testnet for you to send to your friends! :)

Just keep in mind: **testnet tokens do not have monetary value**, and in order to keep the
signal-to-noise ratio high on the server, requests for tokens in other channels will be deleted
without response. Please do not DM Penumbra Labs employees asking for testnet tokens; the correct
venue is the dedicated channel.
