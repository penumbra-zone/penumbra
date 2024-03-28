# Performing chain upgrades

When consensus-breaking changes are made to the Penumbra protocol,
node operators must coordinate upgrading to the new version of the software
at the same time. Penumbra uses a governance proposal for scheduling upgrades
at a specific block height.

## Upgrade process abstractly

At a high level, the upgrade process consists of the following steps:

1. Governance proposal submitted, specifying explicit chain height `n` for halt to occur.
2. Governance proposal passes.
3. Chain reaches specified height `n-1`, nodes stop generating blocks.
4. Manual upgrade is performed on each validator and fullnode:
  1. Prepare migration directory via `pd export`.
  2. Install the new version of pd.
  3. Apply changes to node state via `pd migrate`.
  4. Copy a few files and directories around, clean up CometBFT state.
  5. Restart node.

After the node is restarted on the new version, it should be able to talk to the network again.
Once enough validators with sufficient stake weight have upgraded, the network
will resume generating blocks.

## Genesis time

In order for the chain to start again after the upgrade, all nodes must be using the same genesis information,
including the timestamp for the genesis event. While the `pd migrate` command will create a new `genesis.json` file,
it cannot know the correct genesis start time to use without the operator supplying the `--genesis-start` flag.
The community may choose to specify a start time within the upgrade proposal. If so, all operators must use that value
when performing the migration, as described below. Otherwise, validators must coordinate out of band to agree
on a genesis start time.

Leveraging the governance proposal is the recommended way to solve this problem. If the genesis start time is a value
in the future, then after the upgrade is performed, the node will start, but not process blocks. It will wait
until the `--genesis-start` time is reached, at which point it will resume processing blocks. In this way,
the community of validators can coordinate resumption of chain activity, even when operators perform migrate their ndoes
at slightly different times.

### Testnet 70 -> 71

For the first chain upgrade performed on a Penumbra testnet, use this value for genesis time: `{{ #include ../upgrade_genesis_time_70_71.md }}`.
See an example below for how to supply this value when performing the migration.

## Performing a chain upgrade

The following steps assume that `pd` is using the default home directory of `~/.penumbra/testnet_data/node0/pd`.
If your instance is using a different directory, update the paths accordingly.

1. Stop both `pd` and `cometbft`. Depending on how you run Penumbra, this could mean `sudo systemctl stop penumbra cometbft`.
2. Using the same version of `pd` that was running when the chain halted, prepare an export directory:
   `pd export --home ~/.penumbra/testnet_data/node0/pd --export-directory ~/.penumbra/testnet_data/node0/pd-exported-state`
3. Back up the historical state directory: `mv ~/.penumbra/testnet_data/node0/pd ~/.penumbra/testnet_data/node0/pd-state-backup`
4. Download the latest version of `pd` and install it. Run `pd --version` and confirm you see `{{ #include ../penumbra_version.md }}` before proceeding.

<!--
An example log message emitted by `pd migrate` without providing `--genesis-start`:

    pd::upgrade: no genesis time provided, detecting a testing setup now=2023-12-09T00:08:24.225277473Z`

The value after `now=` is what should be copied. In practice, for testnets, Penumbra Labs will advise on a genesis time
and provide that value in the documentation. Or should we just pick a genesis start ahead of time, and use that for all?
-->
5. Apply the migration: `pd migrate --genesis-start "{{ #include ../upgrade_genesis_time_70_71.md }}" --target-directory ~/.penumbra/testnet_data/node0/pd-exported-state/ --migrate-archive ~/.penumbra/testnet_data/node0/pd-migrated-state-{{ #include ../penumbra_version.md }}.tar.gz`.
   You must use that precise genesis time, otherwise your node will not be able to reach consensus with the rest of the network.
6. Move the migrated state into place: `mkdir ~/.penumbra/testnet_data/node0/pd && mv ~/.penumbra/testnet_data/node0/pd-exported-state/rocksdb ~/.penumbra/testnet_data/node0/pd/`
7. Move the upgrade cometbft state into place: `cp ~/.penumbra/testnet_data/node0/pd-exported-state/genesis.json ~/.penumbra/testnet_data/node0/cometbft/config/genesis.json
   && cp ~/.penumbra/testnet_data/pd-exported-state/priv_validator_state.json ~/.penumbra/testnet_data/node0/cometbft/data/priv_validator_state.json`
8. Then we clean up the old CometBFT state: `find ~/.penumbra/testnet_data/node0/cometbft/data/ -mindepth 1 -maxdepth 1 -type d -exec rm -r {} +`

Finally, restart the node, e.g. `sudo systemctl restart penumbra cometbft`. Check the logs, and you should see the chain progressing
past the halt height `n`.

If you want to host a snapshot for this migration, copy the file
`~/.penumbra/testnet_data/node0/pd-migrated-state-{{ #include ../penumbra_version.md }}.tar.gz` to the appropriate hosting environment,
and inform the users of your validator.
