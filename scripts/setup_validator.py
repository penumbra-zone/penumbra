#!/usr/bin/env python3

import argparse
import json
import os
import shutil
import subprocess
import tempfile

import docker

# Number of blocks per epoch
# We want each epoch around 1 day, so at
# 86400 seconds/day / 10 seconds/block = 8640
EPOCH_DURATION_BLOCKS = 8640


# Fetch the tendermint git repo and build the docker image.
# This is necessary because every architecture (e.g. for our M1 Macs)
# isn't available on DockerHub for this image. git and go build tools
# are required on the host.
def build_tendermint_localnode(testnet_config_dir):
    temp_dir = tempfile.TemporaryDirectory()
    result = subprocess.run(
        [
            "git",
            "clone",
            "git@github.com:tendermint/tendermint.git",
            temp_dir.name,
        ],
        capture_output=True,
        text=True,
    )

    # build the executable
    result = subprocess.run(
        [
            "make build-linux",
        ],
        capture_output=True,
        text=True,
        shell=True,
        cwd=temp_dir.name,
    )

    tendermint_binary = os.path.join(temp_dir.name, "build", "tendermint")
    if not os.path.exists(tendermint_binary):
        raise Exception("error building tendermint binary")

    # copy the executable into the directory used for the testnet configs
    shutil.copy(tendermint_binary, os.path.join(testnet_config_dir, "tendermint"))

    # now the docker image can be built
    client = docker.from_env()
    client.images.build(
        path=os.path.join(temp_dir.name, "networks", "local", "localnode"),
        tag="tendermint/localnode",
    )
    tendermint_localnode_images = client.images.list(name="tendermint/localnode")
    if not tendermint_localnode_images:
        raise Exception("Error building tendermint/localnode image")


# This script will handle configuring a Penumbra docker-compose validator deployment
# by initializing the Tendermint node and patching the genesis.json (stored in the
# Docker volume).
def main(chain_id, testnet_config_dir):
    client = docker.from_env()
    tendermint_localnode_images = client.images.list(name="tendermint/localnode")
    tendermint_binary = os.path.join(testnet_config_dir, "tendermint")
    if not tendermint_localnode_images or not os.path.exists(tendermint_binary):
        # The design of the tendermint localnode image requires the binary to be built
        # on the host and inserted inside, so there's potential for misconfiguration here
        build_tendermint_localnode(testnet_config_dir)

    # The tendermint/localnode image is present so we can create initial testnet
    # configs
    client.containers.run(
        "tendermint/localnode",
        "testnet --config /etc/tendermint/config-template.toml --populate-persistent-peers=false --o . --starting-ip-address 192.167.10.2",
        volumes=[f"{testnet_config_dir}:/tendermint:Z"],
    )

    # Now the testnet_config_dir directory structure should look like:
    #
    # ├── node0
    # │   ├── config
    # │   │   ├── config.toml
    # │   │   ├── genesis.json
    # │   │   ├── node_key.json
    # │   │   └── priv_validator_key.json
    # │   └── data
    # │       └── priv_validator_state.json
    # ├── node1
    # │   ├── config
    # │   │   ├── config.toml
    # │   │   ├── genesis.json
    # │   │   ├── node_key.json
    # │   │   └── priv_validator_key.json
    # │   └── data
    # │       └── priv_validator_state.json
    # ├── node2
    # │   ├── config
    # │   │   ├── config.toml
    # │   │   ├── genesis.json
    # │   │   ├── node_key.json
    # │   │   └── priv_validator_key.json
    # │   └── data
    # │       └── priv_validator_state.json
    # ├── node3
    # │   ├── config
    # │   │   ├── config.toml
    # │   │   ├── genesis.json
    # │   │   ├── node_key.json
    # │   │   └── priv_validator_key.json
    # │   └── data
    # │       └── priv_validator_state.json
    # └── tendermint

    # generate the JSON for the genesis template
    result = subprocess.run(
        [
            "cargo",
            "run",
            "--bin",
            "pd",
            "--",
            "create-genesis-template",
        ],
        capture_output=True,
        text=True,
    )

    genesis_data = json.loads(result.stdout)
    for node in ["node0", "node1", "node2", "node3"]:
        patch_genesis(client, chain_id, genesis_data, testnet_config_dir, node)

    print(
        f"""Testnet configs have been generated and genesis app state injected!
    
    See the magic at: {testnet_config_dir}"""
    )


# This method will patch an existing genesis.json file
# with hardcoded genesis notes for ease of spinning up nodes.
def patch_genesis(
    client: docker.DockerClient, chain_id, genesis_data, testnet_config_dir, node
):
    print(
        "Patching genesis file:",
        os.path.join(testnet_config_dir, node, "config", "genesis.json"),
    )
    # Load the Genesis file as JSON
    existing_genesis = json.load(
        open(os.path.join(testnet_config_dir, node, "config", "genesis.json"))
    )

    existing_genesis["chain_id"] = chain_id

    existing_genesis["app_state"] = {}

    # patch the existing genesis data with our hardcoded data
    existing_genesis["app_state"] = genesis_data

    # write the modified genesis data back
    with open(
        os.path.join(testnet_config_dir, node, "config", "genesis.json"), "w"
    ) as f:
        f.write(json.dumps(existing_genesis, sort_keys=True, indent=4))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate genesis JSON and load into a validator container."
    )
    parser.add_argument(
        "chain_id",
        metavar="c",
        nargs=1,
        help="chain ID",
    )
    parser.add_argument(
        "testnet_config_dir",
        metavar="t",
        nargs=1,
        help="directory to store testnet configs",
    )

    args = parser.parse_args()

    main(args.chain_id[0], args.testnet_config_dir[0])
