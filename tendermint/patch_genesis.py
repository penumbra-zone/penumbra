#!/usr/bin/env python3

import json

# This script will patch an existing genesis.json file
# with hardcoded genesis notes for ease of spinning up nodes.
def main():
    with open("/tendermint/config/genesis.json") as f:
        existing_genesis = json.load(f)

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
    with open("/tendermint/config/genesis.json", "w") as f:
        f.write(json.dumps(existing_genesis))


if __name__ == "__main__":
    main()
