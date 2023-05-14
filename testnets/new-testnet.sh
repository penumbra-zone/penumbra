#!/usr/bin/env bash

set -euo pipefail

echo "#####################################################"
PREVIOUS_TESTNET_DIRECTORY=$(find . -mindepth 1 -type d | sort | tail -n1)
echo "previous testnet directory: $PREVIOUS_TESTNET_DIRECTORY"
PREVIOUS_TESTNET_NUMBER=$(find . -mindepth 1 -type d | wc -l)
echo "previous testnet number: $PREVIOUS_TESTNET_NUMBER"
NEW_TESTNET_NUMBER="0$(echo "1 + $PREVIOUS_TESTNET_NUMBER" | bc)"
echo "new testnet number: $NEW_TESTNET_NUMBER"
NEW_TESTNET_DIRECTORY="$NEW_TESTNET_NUMBER-$(head -"$NEW_TESTNET_NUMBER" < names.txt | tail -1 | tr '[:upper:]' '[:lower:]')"
echo "new testnet directory: $NEW_TESTNET_DIRECTORY"
echo "#####################################################"

echo "Creating new testnet directory $NEW_TESTNET_DIRECTORY..."
mkdir "$NEW_TESTNET_DIRECTORY"
echo "Copying validators from $PREVIOUS_TESTNET_DIRECTORY to $NEW_TESTNET_DIRECTORY"
cp "$PREVIOUS_TESTNET_DIRECTORY/validators.json" "$NEW_TESTNET_DIRECTORY/validators.json"

echo "Setting up allocations for new testnet..."
cut -d , -f 6 < discord_history.csv \
    | tail -n +2 | sort | uniq \
    | grep ^penumbrav \
    | xargs -I{} printf '"1_000__000_000","upenumbra","{}"\n"1_000","test_usd","{}"\n' \
    | cat base_allocations.csv - > "$NEW_TESTNET_DIRECTORY/allocations.csv"

echo "All done!"
