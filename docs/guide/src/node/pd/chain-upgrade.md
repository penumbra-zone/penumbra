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
    1. Install the new version of pd.
    2. Apply changes to `pd` and `cometbft` state via `pd migrate`.
    3. Restart node.

After the node is restarted on the new version, it should be able to talk to the network again.
Once enough validators with sufficient stake weight have upgraded, the network
will resume generating blocks.

## Performing a chain upgrade

Consider performing a backup as a preliminary step during the downtime,
so that your node state is recoverable.

1. Stop both `pd` and `cometbft`. Depending on how you run Penumbra, this could mean `sudo systemctl stop penumbra cometbft`.
2. Download the latest version of `pd` and install it. Run `pd --version` and confirm you see `{{ #include ../../penumbra_version.md }}` before proceeding.
3. Optionally, use `pd export` to create a snapshot of the `pd` state.
4. Apply the migration with `pd migrate --home PD_HOME --comet-home COMETBFT_HOME`.  If using the default home locations (from `pd testnet join`), you can omit the paths and just run `pd migrate`.

Finally, restart the node, e.g. `sudo systemctl restart penumbra cometbft`. Check the logs, and you should see the chain progressing
past the halt height `n`.
