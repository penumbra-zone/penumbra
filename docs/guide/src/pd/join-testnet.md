# Joining a Testnet

We provide instructions for running both fullnode deployments and validator deployments. A
fullnode will sync with the network but will not have any voting power, and will
not be eligible for staking or funding stream rewards. For more information on
what a fullnode is, see the [Tendermint
documentation](https://docs.tendermint.com/v0.34/tendermint-core/using-tendermint.html#adding-a-non-validator).

A regular validator will participate in voting and rewards, if it becomes part
of the consensus set.  Of course, these rewards, like all other testnet tokens,
have no value.

## Joining as a fullnode

To join a testnet as a fullnode, check out the tag for the current testnet, run
`pd testnet join` to generate configs, then use those configs to run `pd` and
`tendermint`. In more detail:

### Resetting state

First, reset the testnet data from any prior testnet you may have joined:

```shell
cargo run --bin pd --release -- testnet unsafe-reset-all
```

This will delete the entire testnet data directory.

### Generating configs

Next, generate a set of configs for the current testnet:

```shell
cargo run --bin pd --release -- testnet join --external-address IP_ADDRESS:26656 --moniker MY_NODE_NAME
```

where `IP_ADDRESS` (like `1.2.3.4`) is the public IP address of the node you're running,
and `MY_NODE_NAME` is a moniker identifying your node. Other peers will try to connect
to your node over port 26656/TCP.

If your node is behind a firewall or not publicly routable for some other reason,
skip the `--external-address` flag, so that other peers won't try to connect to it.
You can also skip the `--moniker` flag to use a randomized moniker instead of selecting one.

This command fetches the genesis file for the current testnet, and writes
configs to a testnet data directory (by default, `~/.penumbra/testnet_data`).
If any data exists in the testnet data directory, this command will fail.  See
the section above on resetting node state.

### Running `pd` and `tendermint`

Next, run `pd` with the `--home` parameter pointed at the correct part of the
testnet data directory.  It's useful to set the `RUST_LOG` environment variable
to get information about what it's doing:

```shell
export RUST_LOG="warn,pd=debug,penumbra=debug" # or some other logging level
```

```shell
cargo run --bin pd --release -- start --home ~/.penumbra/testnet_data/node0/pd
```

Then (perhaps in another terminal), run Tendermint, also specifying `--home`:

```shell
tendermint start --home ~/.penumbra/testnet_data/node0/tendermint
```

Alternatively, `pd` and `tendermint` can be orchestrated with `docker-compose`:

```shell
cd deployments/compose/
docker-compose pull
docker-compose up --abort-on-container-exit
```

or via systemd:

```
cd deployments/systemd/
sudo cp *.service /etc/systemd/system/
# edit service files to customize for your system
sudo systemctl daemon-reload
sudo systemctl restart penumbra tendermint
```

## Joining as a validator

After starting your node, as above, you should now be participating in the
network as a fullnode. However your validator won't be visible to the chain yet,
as the definition hasn't been uploaded.

### Validator Definitions (Penumbra)

A validator definition contains fields defining metadata regarding your
validator as well as funding streams, which are Penumbra's analogue to validator
commissions.

The root of a validator's identity is their *identity key*.  Currently, `pcli`
reuses the spend authorization key in whatever wallet is active as the
validator's identity key.  This key is used to sign validator definitions that
update the configuration for a validator.

#### Creating a template definition

To create a template configuration, use `pcli validator definition template`:

```shell
$ cargo run --release --bin pcli -- validator definition template \
    --tendermint-validator-keyfile ~/.penumbra/testnet_data/node0/tendermint/config/priv_validator_key.json \
    --file validator.toml
$ cat validator.toml
# This is a template for a validator definition.
#
# The identity_key and governance_key fields are auto-filled with values derived
# from this wallet's account.
#
# You should fill in the name, website, and description fields.
#
# By default, validators are disabled, and cannot be delegated to. To change
# this, set `enabled = true`.
#
# Every time you upload a new validator config, you'll need to increment the
# `sequence_number`.

sequence_number = 0
enabled = false
name = ''
website = ''
description = ''
identity_key = 'penumbravalid1kqrecmvwcc75rvg9arhl0apsggtuannqphxhlzl34vfamp4ukg9q87ejej'
governance_key = 'penumbragovern1kqrecmvwcc75rvg9arhl0apsggtuannqphxhlzl34vfamp4ukg9qus84v5'

[consensus_key]
type = 'tendermint/PubKeyEd25519'
value = 'HDmm2FmJhLHxaKPnP5Fw3tC1DtlBx8ETgTL35UF+p6w='

[[funding_stream]]
recipient = 'penumbrav2t1cntf73e36y3um4zmqm4j0zar3jyxvyfqxywwg5q6fjxzhe28qttppmcww2kunetdp3q2zywcakwv6tzxdnaa3sqymll2gzq6zqhr5p0v7fnfdaghrr2ru2uw78nkeyt49uf49q'
rate_bps = 100

[[funding_stream]]
recipient = "DAO"
rate_bps = 100
```

and adjust the data like the name, website, description, etc as desired.

The `enabled` field can be used to enable/disable your validator without facing slashing
penalties. Disabled validators can not appear in the active validator set and are ineligible for
rewards.

This is useful if, for example, you know your validator will not be online for a period of time,
and you want to avoid an uptime violation penalty. If you are uploading your validator for the
first time, you will likely want to start with it disabled until your Tendermint & `pd`
instances have caught up to the consensus block height.

Note that by default the `enabled` field is set to false and will need to be
enabled in order to activate one's validator.

In the default template, there is a funding stream declared to [contribute funds to the
DAO](../pcli/governance.md#contributing-to-the-dao). This is not required, and may be altered or
removed if you wish.

#### Setting the consensus key

In the command above, the `--tendermint-validator-keyfile` flag was used to instruct
`pcli` to import the consensus key for the Tendermint identity. This works well
if `pcli` and `pd` are used on the same machine. If you are running them in separate
environments, you can omit the flag, and `pd` will generate a random key in the template.
You must then **manually update the `consensus_key`**. You can get the correct value
for `consensus_key` from your `tendermint` configs:

```shell
$ grep -A3 pub_key ~/.penumbra/testnet_data/node0/tendermint/config/priv_validator_key.json
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "Fodjg0m1kF/6uzcAZpRcLJswGf3EeNShLP2A+UCz8lw="
  },
```

Copy the string in the `value` field and paste that into your `validator.toml`,
as the `value` field under the `[consensus_key]` heading.

#### Configuring funding streams

Unlike the Cosmos SDK, which has validators specify a commission percentage that
goes to the validator, Penumbra uses *funding streams*, a list of pairs of
commission amounts and addresses.  This design allows validators to dedicate
portions of their commission non-custodially -- for instance, a validator could
declare some amount of commission to cover their operating costs, and another
that would be sent to an address controlled by a DAO.

## Uploading a definition

After setting up metadata, funding streams, and the correct consensus key in
your `validator.toml`, you can upload it to the chain:

```console
cargo run --release --bin pcli -- validator definition upload --file validator.toml
```

And verify that it's known to the chain:

```console
cargo run --release --bin pcli -- query validator list -i
```

However your validator doesn't have anything delegated to it and will remain in
an `Inactive` state until it receives enough delegations to place it in the
active set of validators.

## Delegating to your validator

First find your validator's identity key:

```console
cargo run --release --bin pcli -- validator identity
```

And then delegate some amount of `penumbra` to it:

```console
cargo run --release --bin pcli -- tx delegate 1penumbra --to penumbravalid1g2huds8klwypzczfgx67j7zp6ntq2m5fxmctkf7ja96zn49d6s9qz72hu3
```

You should then see your balance of `penumbra` decreased and that you have received some amount of delegation tokens for your validator:

```console
cargo run --release --bin pcli view balance
```

Voting power will be calculated on the next epoch transition after your
delegation takes place.  Assuming that your delegation was enough to place your
validator in the top N validators by voting power, it should appear in the
validator list as `Active` after the next epoch transition.  The epoch duration
and the active validator limit are chain parameters, and will vary by
deployment.  You can find the values in use for the current chain in its
`genesis.json` file.

## Updating your validator

First fetch your existing validator definition from the chain:

```console
cargo run --release --bin pcli -- validator definition fetch --file validator.toml
```

Then make any changes desired and **make sure to increase by `sequence_number` by at least 1!**
The `sequence_number` is a unique, increasing identifier for the version of the validator definition.

After updating the validator definition you can upload it again to update your validator metadata on-chain:

```console
cargo run --release --bin pcli -- validator definition upload --file validator.toml
```
