# Joining a Testnet

We provide instructions for running both fullnode deployments and validator deployments. A
fullnode will sync with the network but will not have any voting power, and will
not be eligible for staking or funding stream rewards. For more information on
what a fullnode is, see the [CometBFT
documentation](https://docs.cometbft.com/v0.37/core/using-cometbft#adding-a-non-validator).

A regular validator will participate in voting and rewards, if it becomes part
of the consensus set.  Of course, these rewards, like all other testnet tokens,
have no value.

## Generating configs

To join a testnet as a fullnode, [install the most recent version of `pd`](install.md), run
`pd testnet join` to generate configs, then use those configs to run `pd` and
`cometbft`.

```shell
pd testnet join \
    --moniker MY_NODE_NAME \
    --external-address IP_ADDRESS:26656 \
    --archive-url "https://snapshots.penumbra.zone/testnet/pd-migrated-state-77-78.tar.gz"
```

where `MY_NODE_NAME` is a moniker identifying your node, and `IP_ADDRESS` (like `1.2.3.4`)
is the public IP address of the node you're running. Other peers will try to connect
to your node over port `26656/TCP`. Finally, the `--archive-url` flag will fetch
a tarball of historical blocks, so that your newly joining node can understand transactions
that occurred prior to the most recent chain upgrade.

If your node is behind a firewall or not publicly routable for some other reason,
skip the `--external-address` flag, so that other peers won't try to connect to it.
You can also skip the `--moniker` flag to use a randomized moniker instead of selecting one.

This command fetches the genesis file for the current testnet, and writes
configs to a testnet data directory (by default, `~/.penumbra/testnet_data`).
If any data exists in the testnet data directory, this command will fail.  See
the section above on resetting node state.

### Running `pd` and `cometbft`

Copy the systemd service configs into place from the [project git repo](https://github.com/penumbra-zone/penumbra):

```
cd deployments/systemd/
sudo cp *.service /etc/systemd/system/
# edit service files to customize for your system
sudo systemctl daemon-reload
sudo systemctl restart penumbra cometbft
```

In particular, if you have DNS configured for your node, you should edit the `ExecStart` line for `pd`
to use the `--grpc-auto-https` option.

### Resetting state

If you have previously joined a network before, and want to purge those configs,
use:

```shell
pd testnet unsafe-reset-all
```

This will delete the entire testnet data directory, after which you can re-join.
You should only run this command after stopping `pd` and `cometbft`.
