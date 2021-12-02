#!/usr/bin/env python3

import argparse
import json
import subprocess
import tempfile

import docker


# This script will handle configuring a Penumbra docker-compose validator deployment
# by initializing the Tendermint node and patching the genesis.json (stored in the
# Docker volume).
def main(genesis_csv_file):
    client = docker.from_env()

    # generate the JSON
    result = subprocess.run(
        [
            "cargo",
            "run",
            "--bin",
            "pd",
            "--",
            "create-genesis",
            "penumbra-valetudo",
            "--file",
            genesis_csv_file,
        ],
        capture_output=True,
        text=True,
    )

    genesis_data = json.loads(result.stdout)
    patch_genesis(client, genesis_data)

    # restart the containers to pick up changes
    subprocess.run(["docker-compose", "restart"])


# This method will patch an existing genesis.json file
# with hardcoded genesis notes for ease of spinning up nodes.
def patch_genesis(client: docker.DockerClient, genesis_data):
    temp_dir = tempfile.TemporaryDirectory()

    # Load the Genesis file as JSON
    existing_genesis = json.loads(
        client.containers.run(
            "alpine",
            "/source/config/genesis.json",
            stderr=True,
            remove=True,
            entrypoint="cat",
            volumes=[f"{temp_dir.name}:/dest", "penumbra_tendermint_data:/source"],
        ).decode("utf-8")
    )

    # patch the existing genesis data with our hardcoded notes
    existing_genesis["app_state"] = genesis_data

    # write the modified genesis data back
    with open(f"{temp_dir.name}/genesis.json", "w") as f:
        f.write(json.dumps(existing_genesis))

    # store the modified genesis file within the volume
    client.containers.run(
        "alpine",
        "/source/genesis.json /dest/config/genesis.json",
        stderr=True,
        remove=True,
        entrypoint="cp",
        volumes=[f"{temp_dir.name}:/source", "penumbra_tendermint_data:/dest"],
    ).decode("utf-8")

    temp_dir.cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate genesis JSON and load into a validator container."
    )
    parser.add_argument(
        "genesis_file",
        metavar="f",
        nargs=1,
        help="which file contains the CSV genesis data",
    )

    args = parser.parse_args()

    main(args.genesis_file[0])
