# Testing IBC

This guide explains how to work with IBC functionality
while developing Penumbra.

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

## Making Osmosis -> Penumbra transfers, via rly

Transferring from Osmosis to Penumbra requires making an Osmosis transaction.
The `osmosisd` CLI tooling unfortunately does not work for IBC transfers.
To move funds from a Penumbra chain to an Osmosis testnet, use the `rly` binary
from the [cosmos/relayer repo](https://github.com/cosmos/relayer). Then run:

```
# inside the penumbra repo:
cd deployments/relayer
# refresh the chain id for local devnet:
./generate-configs local
rly config init --memo "PenumbraIBC"
rly chains add -f configs/penumbra-local.json
rly chains add -f configs/osmosis-testnet.json
# use the seed phrase from 1password for the osmosis key:
rly keys restore osmosis-testnet default "SEED PHRASE"
rly keys add penumbra-local default
# create an IBC path between the two chains
rly paths add $PENUMBRA_DEVNET_CHAIN_ID osmo-test-5 penumbra-osmosis-dev

# finally, make the transfer
rly transact transfer osmosis-testnet penumbra-local 100uosmo penumbrav2t1jp4pryqqmh65pq8e7zwk6k2674vwhn4qqphxjk0vukxln0crmp2tdld0mhavuyrspwuajnsk5t5t33u2auxvheunr7qde4l068ez0euvtu08z7rwj6shlh64ndz0wvz7mfqdcd channel-1675 -y 10000 -c 2h
2023-09-17T20:19:47.510916Z	info	Successfully sent a transfer	{"src_chain_id": "osmo-test-5", "dst_chain_id": "penumbra-testnet-tethys-8777cb20", "send_result": {"successful_src_batches": 1, "successful_dst_batches": 0, "src_send_errors": "<nil>", "dst_send_errors": "<nil>"}}
2023-09-17T20:19:47.510938Z	info	Successful transaction	{"provider_type": "cosmos", "chain_id": "osmo-test-5", "packet_src_channel": "channel-1675", "packet_dst_channel": "channel-0", "gas_used": 117670, "fees": "3717uosmo", "fee_payer": "osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488", "height": 2720869, "msg_types": ["/ibc.applications.transfer.v1.MsgTransfer"], "tx_hash": "67C55AD4FC6855579C2B4B421F8F02B2B2BFE6BA0D5C1553C13BBB7DFFAD781D"}
```

You can view account history for the shared Osmosis testnet account here:
[https://testnet.mintscan.io/osmosis-testnet/account/osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488](https://testnet.mintscan.io/osmosis-testnet/account/osmo1kh0fwkdy05yp579d8vczgharkcexfw582zj488)

## Updating Hermes config for a new testnet
On every release of a new Penumbra testnet, we must update the Hermes relayer to establish
a channel between it and target counterparty test chains.

1. [Checkout the Penumbra fork of Hermes](https://github.com/penumbra-zone/hermes),
and build it with `cargo build --release`.
3. Edit the `config-osmosis-testnet.toml` file to use the chain id of the new Penumbra testnet, e.g. `penumbra-testnet-dione`.
4. Add Osmosis key material to Hermes. Look up the Osmosis recovery phrase
stored in shared 1Password, then:
```bash
echo "SEED PHRASE" > ./mnemonic
cargo run --release --bin hermes -- --config config-osmosis-testnet.toml keys add --chain osmo-test-5 --mnemonic-file ./mnemonic
```
5. Create a new channel for this testnet:
```bash
cargo run --release --bin hermes -- --config config-osmosis-testnet.toml create channel --a-chain $PENUMBRA_TESTNET_CHAIN_ID --b-chain osmo-test-5 --a-port transfer --b-port transfer --new-client-connection
```
Hermes will run for a while, emit channel info, and then exit.
6. Run Hermes: `cargo run --release --bin hermes -- --config config-osmosis-testnet.toml start`

Use the [IBC user docs](../pcli/transaction.md#ibc-withdrawals) to make a test transaction,
to ensure that relaying is working. In the future, we should consider posting the newly created
channel to the IBC docs guide, so community members can use it.
