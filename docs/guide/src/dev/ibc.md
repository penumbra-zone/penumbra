# Testing IBC

This guide explains how to work with IBC functionality
while developing Penumbra.

## Making Penumbra -> Osmosis outbound transfers, via pcli
See the [IBC user docs](../pcli/transaction.md#ibc-withdrawals) for how to use
`pcli` to make an outbound IBC withdrawal, to a different testnet.

## Making Osmosis -> Penumbra inbound transfers, via hermes

Transferring from Osmosis to Penumbra requires making an Osmosis transaction.
The `osmosisd` CLI tooling unfortunately does not work for IBC transfers.
To move funds from a Penumbra chain to an Osmosis testnet, use the `hermes` binary
from the [Penumbra fork](https://github.com/penumbra-zone/hermes). What you'll need:

* a local checkout of Hermes
* your own osmosis wallet, with funds from the testnet faucet
* channel info for both chains (consult `pcli query ibc channels`)
* a penumbra address

You should use your own Osmosis wallet, with funds from the testnet faucet,
and configure Hermes locally on your workstation with key material. Do *not*
reuse the Hermes relayer instance, as sending transactions from its wallets
while it's relaying can break things.

```bash
# Hop into the hermes repo and build it:
cargo build --release

# Edit `config-penumbra-osmosis.toml` with your Penumbra wallet SpendKey,
# and make sure the Penumbra chain id is correct.
# Add your osmosis seed phrase to the file `mnemonic-osmosis-transfer`,
# then import it:
cargo run --release --bin hermes -- \
    --config config-penumbra-osmosis.toml keys add \
    --chain osmo-test-5 --mnemonic-file ./mnemonic-osmosis-transfer

# Then run a one-off command to trigger an outbound IBC transfer,
# from Osmosis to Penumbra:
cargo run --release --bin hermes -- \
    --config ./config-penumbra-osmosis.toml tx ft-transfer \
    --dst-chain <PENUMBRA_CHAIN_ID> --src-chain osmo-test-5 --src-port transfer \
    --src-channel <CHANNEL_ID_ON_OSMOSIS_CHAIN> --denom uosmo --amount 100 \
    --timeout-height-offset 10000000 --timeout-seconds 10000000 \
    --receiver <PENUMBRA_ADDRESS>
```

You can view account history for the shared Osmosis testnet account here:
[https://www.mintscan.io/osmosis-testnet/account/osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488](https://www.mintscan.io/osmosis-testnet/account/osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488).
Change the address at the end of the URL to your account to confirm that your test transfer worked.

## Updating Hermes config for a new testnet
See the [procedure in the wiki](https://github.com/penumbra-zone/penumbra/wiki/Updating-Hermes)
for up to date information.

Use the [IBC user docs](../pcli/transaction.md#ibc-withdrawals) to make a test transaction,
to ensure that relaying is working. In the future, we should consider posting the newly created
channel to the IBC docs guide, so community members can use it.

## Working with a local devnet

<!--
The original source of the local devnet docs is this PR comment:
https://github.com/penumbra-zone/penumbra/pull/3043/#issuecomment-1722554083
You may want to consult that PR for additional context.
-->

Use this approach while fixing bugs or adding features.
Be aware that creating a new channel on the public Osmosis testnet
creates lingering state on that counterparty chain. Be respectful.

1. [Create a devnet](./devnet-quickstart.md). Make note of the randomly
generated chain id emitted in the logs, as we'll need it to configure Hermes.
2. [Checkout the Penumbra fork of Hermes](https://github.com/penumbra-zone/hermes),
and build it with `cargo build --release`.
3. Edit the `config-devnet-osmosis.toml` file to use the chain id for your newly created devnet.
4. Add Osmosis key material to Hermes. Look up the Osmosis recovery phrase
stored in shared 1Password, then:
```bash
echo "SEED PHRASE" > ./mnemonic
cargo run --release --bin hermes -- --config config-devnet-osmosis.toml keys add --chain osmo-test-5 --mnemonic-file ./mnemonic
```
5. Create a new channel for this devnet:
```bash
cargo run --release --bin hermes -- --config config-devnet-osmosis.toml create channel --a-chain $PENUMBRA_DEVNET_CHAIN_ID --b-chain osmo-test-5 --a-port transfer --b-port transfer --new-client-connection
```
Hermes will run for a while, emit channel info, and then exit.
6. Finally, run Hermes: `cargo run --release --bin hermes -- --config config-devnet-osmosis.toml start`

You may see a spurious error about "signature key not found: penumbra-wallet: cannot find key file".
Ignore that error: we haven't implemented fees yet, so no Penumbra keys are required in Hermes.
Hermes will emit a summary of the channel info, something like:

```
# Chain: penumbra-testnet-tethys-8777cb20
  - Client: 07-tendermint-0
  - Client: 07-tendermint-1
    * Connection: connection-0
      | State: OPEN
      | Counterparty state: OPEN
      + Channel: channel-0
        | Port: transfer
        | State: OPEN
        | Counterparty: channel-1675
# Chain: osmo-test-5
  - Client: 07-tendermint-1029
    * Connection: connection-939
      | State: OPEN
      | Counterparty state: OPEN
      + Channel: channel-1675
        | Port: transfer
        | State: OPEN
        | Counterparty: channel-0
```

Make note of the channels on both the primary (Penumbra devnet) and counterparty (Osmosis testnet) chains.
You can use those values to send funds from the Penumbra devnet to the counterparty:

```bash
cargo run --release --bin pcli -- -n http://localhost:8080 view reset
# check what funds are available
cargo run --release --bin pcli -- -n http://localhost:8080 view balance
cargo run --release --bin pcli -- -n http://localhost:8080 tx withdraw --to osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488 --channel 0 --timeout-height 5-2900000 100penumbra
```

See the [IBC pcli docs](../pcli/transaction.md#ibc-withdrawals) for more details.
