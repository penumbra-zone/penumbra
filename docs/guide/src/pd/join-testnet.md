# Joining a Testnet

We provide instructions for running both fullnode deployments and validator deployments. A
fullnode will sync with the network but will not have any voting power, and will
not be eligible for staking or funding stream rewards. For more information on
what a fullnode is, see the [CometBFT
documentation](https://docs.cometbft.com/v0.37/core/using-cometbft#adding-a-non-validator).

A regular validator will participate in voting and rewards, if it becomes part
of the consensus set.  Of course, these rewards, like all other testnet tokens,
have no value.

## Joining as a fullnode

To join a testnet as a fullnode, [install the most recent version of `pd`](install.md), run
`pd testnet join` to generate configs, then use those configs to run `pd` and
`cometbft`. In more detail:

### Resetting state

First, reset the testnet data from any prior testnet you may have joined:

```shell
pd testnet unsafe-reset-all
```

This will delete the entire testnet data directory.

### Generating configs

Next, generate a set of configs for the current testnet:

<!--
### Begin join customization

The following section describes how to join a testnet chain *which has never upgraded*.
Once a chain upgrade occurs, a new-joining node must have access to an archive
of historical, migrated state. When we upgrade the chain, we should update these
docs to switch to the archive-url version:

```shell
pd testnet join --external-address IP_ADDRESS:26656 --moniker MY_NODE_NAME \
    --archive-url "https://snapshots.penumbra.zone/testnet/pd-archived-stated-xxxxx.tar.gz
```

where `IP_ADDRESS` (like `1.2.3.4`) is the public IP address of the node you're running,
and `MY_NODE_NAME` is a moniker identifying your node. Other peers will try to connect
to your node over port `26656/TCP`. Finally, the `--archive-url` flag will fetch
a tarball of historical blocks, so that your newly joining node can understand transactions
that occurred prior to the most recent chain upgrade.
-->

```shell
pd testnet join --external-address IP_ADDRESS:26656 --moniker MY_NODE_NAME
```

where `IP_ADDRESS` (like `1.2.3.4`) is the public IP address of the node you're running,
and `MY_NODE_NAME` is a moniker identifying your node. Other peers will try to connect
to your node over port `26656/TCP`.
<!--
### End join customization
-->

If your node is behind a firewall or not publicly routable for some other reason,
skip the `--external-address` flag, so that other peers won't try to connect to it.
You can also skip the `--moniker` flag to use a randomized moniker instead of selecting one.

This command fetches the genesis file for the current testnet, and writes
configs to a testnet data directory (by default, `~/.penumbra/testnet_data`).
If any data exists in the testnet data directory, this command will fail.  See
the section above on resetting node state.

### Running `pd` and `cometbft`

Next, run `pd`:

```shell
pd start
```

Then (perhaps in another terminal), run CometBFT, specifying `--home`:

```shell
cometbft start --home ~/.penumbra/testnet_data/node0/cometbft
```

Alternatively, `pd` and `cometbft` can be orchestrated with `docker-compose`:

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
sudo systemctl restart penumbra cometbft
```

See the [`deployments/`](https://github.com/penumbra-zone/penumbra/tree/{{ #include ../penumbra_version.md }}/deployments)
directory for more examples on configuration scripts.
