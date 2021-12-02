# Testnet Genesis Readme

## names.txt

`names.txt` contains the list of testnet names! They're moons, of Jupiter.

## $moon.csv

Each testnet will have an associated `csv` file containing the block of
initialization data to be supplied to `create-genesis`.

The `scripts/setup_validator.py` script takes as a command line argument
one of the `$moon.csv` files, and will handle generating the appropriate
Genesis JSON and loading it into a running validator Docker container.