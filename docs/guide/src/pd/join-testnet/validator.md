# Validator Deployment

Every validator is also a full node, so first follow the configuration
instructions in [the previous section](./fullnode.md).

To run a validator, you'll need to do some Tendermint-related configuration to
tell Tendermint to run a validator, then some Penumbra-related configuration to
declare your validator to the network.

## Validator Configuration (Tendermint)

For a validator, you'll **need to make sure that tendermint is set to validator
mode** in the Tendermint `config.toml`:
```toml
mode = "validator"
```

Also, make sure that you followed the instructions in [the previous section](./fullnode.md) for pulling in Penumbra's tendermint configuration.

After starting your node, as in [the previous section](./fullnode.md), you
should now be participating in the network as a fullnode. However your validator
won't be visible to the chain yet, as the definition hasn't been uploaded.

## Validator Definitions (Penumbra)

A validator definition contains fields defining metadata regarding your
validator as well as initial funding streams.

The root of a validator's identity is their *identity key*.  Currently, `pcli`
reuses the spend authorization key in whatever wallet is active as the
validator's identity key.  This key is used to sign validator definitions that
update the configuration for a validator.

#### Creating a template definition

To create a template configuration, use `pcli validator template-definition`:
```console
\$ cargo run --release --bin pcli -- validator template-definition --file validator.json
\$ cat validator.json
{
  "identity_key": "penumbravalid1g2huds8klwypzczfgx67j7zp6ntq2m5fxmctkf7ja96zn49d6s9qz72hu3",
  "consensus_key": "Fodjg0m1kF/6uzcAZpRcLJswGf3EeNShLP2A+UCz8lw=",
  "name": "",
  "website": "",
  "description": "",
  "enabled": false,
  "funding_streams": [
    {
      "address": "penumbrav1t1mw8270qtpgjy628fg97p2px45e860jtlw0nl3w5y7vq67qx697py9t8ppp3mhwfxv8kegg8wuny64nf60z966krx85cqznjpshqtngffpwnywtzqjklkg3qh7anxk368ywac9l",
      "rate_bps": 100
    }
  ],
  "sequence_number": 0
}
```
and adjust the data like the name, website, description, etc as desired.

The `enabled` field can be used to enable/disable your validator without facing slashing 
penalties. Disabled validators can not appear in the active validator set and are ineligible for 
rewards.

This is useful if, for example, you know your validator will not be online for a period of time, 
and you want to avoid an uptime violation penalty. If you are uploading your validator for the 
first time, you will likely want to start with it disabled until your Tendermint & `pd` 
instances have caught up to the consensus block height.

#### Setting the consensus key

By default `template-definition` will use a random consensus key that you won't have access to. Make sure to **set the `consensus_key` correctly** otherwise your instance of `tendermint` won't be using the key expected in the validator definition. You can get the correct value for `consensus_key` from your `tendermint` configs:

```console
\$ grep -A3 pub_key ~/.tendermint/config/priv_validator_key.json
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "Fodjg0m1kF/6uzcAZpRcLJswGf3EeNShLP2A+UCz8lw="
  },
```

Note: if you can't find `priv_validator_key.json`, assure that you have set
`validator = true` in the Tendermint `config.toml`, as described above, and
restarted Tendermint after doing so.

#### Configuring funding streams

Unlike the Cosmos SDK, which has validators specify a commission percentage that
goes to the validator, Penumbra uses *funding streams*, a list of pairs of
commission amounts and addresses.  This design allows validators to dedicate
portions of their commission non-custodially -- for instance, a validator could
declare some amount of commission to cover their operating costs, and another
that would be sent to an address controlled by a DAO.

## Uploading a definition

After setting up metadata, funding streams, and the correct consensus key in your `validator.json`, you can upload it to the chain:

```console
cargo run --release --bin pcli -- validator upload-definition --file validator.json
```

And verify that it's known to the chain:

```console
cargo run --release --bin pcli -- stake list-validators -i
```

However your validator doesn't have anything delegated to it and will remain in an `Inactive` state until it receives enough delegations
to place it in the active set of validators.

## Delegating to your validator

First find your validator's identity key:

```console
cargo run --release --bin pcli -- stake list-validators -i
```

And then delegate some amount of `penumbra` to it:

```console
cargo run --release --bin pcli -- stake delegate 1penumbra --to penumbravalid1g2huds8klwypzczfgx67j7zp6ntq2m5fxmctkf7ja96zn49d6s9qz72hu3
```

You should then see your balance of `penumbra` decreased and that you have received some amount of delegation tokens for your validator:

```console
cargo run --release --bin pcli -- balance
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
cargo run --release --bin pcli -- validator fetch-definition penumbravalid1dj3mgmje7z9mwu6rl9rplue2neqlc4zwls9a7w88vscwv88ltyqs8v6g9x --file validator.json
```

Then make any changes desired and **make sure to increase by `sequence_number` by at least 1!**
The `sequence_number` is a unique, increasing identifier for the version of the validator definition.

After updating the validator definition you can upload it again to update your validator metadata on-chain:

```console
cargo run --release --bin pcli -- validator upload-definition --file validator.json
```
