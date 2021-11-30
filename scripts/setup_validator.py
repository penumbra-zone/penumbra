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
            "diversifier": "b261d7629fcf8910ac5b1a",
            "amount": 100,
            "note_blinding": "eb5537f7ea3f0769f0d97e0bbeafc79ecfc4f56b21d45f80cade50d59562f811",
            "asset_denom": "pen",
            "transmission_key": "98d1a1b19ffed22c59242140a46e042713770b2930a95ea9f080c612409fef02",
        },
        {
            "diversifier": "b261d7629fcf8910ac5b1a",
            "amount": 1,
            "note_blinding": "19cb10699b7aa33fcbc1dd4438c3fb6b02af4f880d39ce98075aa241ee5a2b04",
            "asset_denom": "tungsten_cube",
            "transmission_key": "98d1a1b19ffed22c59242140a46e042713770b2930a95ea9f080c612409fef02",
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
