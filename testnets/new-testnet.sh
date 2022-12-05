#!/usr/bin/env bash

set -euo pipefail

PREVIOUS_TESTNET_DIRECTORY=$(ls -F | grep '/$' | tail -1)
PREVIOUS_TESTNET_NUMBER=$(ls -F | grep '/$' | wc -l)
NEW_TESTNET_NUMBER="0$(echo "1 + $PREVIOUS_TESTNET_NUMBER" | bc)"
NEW_TESTNET_DIRECTORY="$NEW_TESTNET_NUMBER-$(head -$NEW_TESTNET_NUMBER < names.txt | tail -1 | tr '[:upper:]' '[:lower:]')"

echo "Creating new testnet directory $NEW_TESTNET_DIRECTORY..."

mkdir "$NEW_TESTNET_DIRECTORY"
cp "$PREVIOUS_TESTNET_DIRECTORY/validators.json" "$NEW_TESTNET_DIRECTORY/validators.json"

echo "Setting up allocations for new testnet..."

cut -d , -f 6 < discord_history.csv \
    | tail -n +2 | sort | uniq \
    | sed -e 's/^/"1_000__000_000","upenumbra","/; s/$/"/' \
    | cat base_allocations.csv - > "$NEW_TESTNET_DIRECTORY/allocations.csv"

echo "All done!"
