#!/usr/bin/env python3

import json
import subprocess
import tempfile

import docker


# This script will handle configuring a Penumbra docker-compose validator deployment
# by initializing the Tendermint node and patching the genesis.json (stored in the
# Docker volume).
def main():
    client = docker.from_env()

    patch_genesis(client)

    # restart the containers to pick up changes
    subprocess.run(["docker-compose", "restart"])


# This method will patch an existing genesis.json file
# with hardcoded genesis notes for ease of spinning up nodes.
def patch_genesis(client: docker.DockerClient):
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
    existing_genesis["app_state"] = [
        {
            "diversifier": "b050dbfc4e86ac4b2b09a1",
            "amount": 100,
            "note_blinding": "93f7245c0e0265338ed54db574462d16a366187d3f2ff361aa94ecddadfbb103",
            "asset_denom": "pen",
            "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503",
        },
        {
            "diversifier": "b050dbfc4e86ac4b2b09a1",
            "amount": 1,
            "note_blinding": "7793daf7ac4ef421d6ad138675180b37b866cc5ca0297a846fb9301d9deb2c0d",
            "asset_denom": "tungsten_cube",
            "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503",
        },
    ]

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
    main()
