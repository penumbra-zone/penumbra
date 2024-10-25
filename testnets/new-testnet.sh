#!/usr/bin/env bash

set -euo pipefail

echo "#####################################################"
PREVIOUS_TESTNET_DIRECTORY=$(find . -mindepth 1 -type d | sort | tail -n1)
echo "previous testnet directory: $PREVIOUS_TESTNET_DIRECTORY"
PREVIOUS_TESTNET_NUMBER="$(find . -mindepth 1 -type d -exec basename {} \; | tail -n 1 | grep -Po '^\d+')"
echo "previous testnet number: $PREVIOUS_TESTNET_NUMBER"
NEW_TESTNET_NUMBER="0$(echo "1 + $PREVIOUS_TESTNET_NUMBER" | bc)"
echo "new testnet number: $NEW_TESTNET_NUMBER"
NEW_TESTNET_DIRECTORY="$NEW_TESTNET_NUMBER-$(head -"$NEW_TESTNET_NUMBER" < names.txt | tail -1 | tr '[:upper:]' '[:lower:]')"
echo "new testnet directory: $NEW_TESTNET_DIRECTORY"
echo "#####################################################"

echo "Creating new testnet directory $NEW_TESTNET_DIRECTORY..."
mkdir "$NEW_TESTNET_DIRECTORY"
if [[ -e "$PREVIOUS_TESTNET_DIRECTORY/validators.json" ]]; then
    echo "Copying validators from $PREVIOUS_TESTNET_DIRECTORY to $NEW_TESTNET_DIRECTORY"
    cp -v "$PREVIOUS_TESTNET_DIRECTORY/validators.json" "$NEW_TESTNET_DIRECTORY/validators.json"
else
    echo "Using default CI validator config"
    # We inspect the validators config and pluck the first entry out, for a solo-validator setup.
    # TODO: update pd to take an `--n-validators` arg so this is dynamic.
    jq '.[0]' "validators-ci.json" | jq -s > "$NEW_TESTNET_DIRECTORY/validators.json"
fi

echo "Setting up allocations for new testnet..."
# Truncate file, set CSV headers.
echo "amount,denom,address" > base_allocations.csv
# Read in base_addresses for team allocations, assign same amount for everyone.
while read -r a ; do
    cat <<EOM >> base_allocations.csv
1_000_000__000_000,upenumbra,$a
20_000,gm,$a
20_000,gn,$a
10_000,pizza,$a
100,cube,$a
500_000,test_usd,$a
EOM
done < base_addresses.txt

# The Galileo bot has multiple addresses since
# https://github.com/penumbra-zone/galileo/pull/72.
# We'll assign ample creds to each.
while read -r a; do
    cat <<EOM >> base_allocations.csv
5_000_000__000_000,upenumbra,$a
50_000,gm,$a
50_000,gn,$a
25_000,pizza,$a
250,cube,$a
1_000_000,test_usd,$a
10_000,nala,$a
EOM
done < galileo_addresses.txt

# The Osiris bot has very large allocations, so it can MM.
while read -r a; do
    cat <<EOM >> base_allocations.csv
10_000_000__000_000,upenumbra,$a
10_000_000,gm,$a
10_000_000,gn,$a
10_000_000,pizza,$a
10_000_000,cube,$a
10_000_000,test_usd,$a
10_000_000,test_btc,$a
10_000_000,test_eth,$a
10_000_000,test_atom,$a
10_000_000,test_osmo,$a
EOM
done < <(cut -d' ' -f1 "osiris_addresses.txt")

# The integration tests expect precise allocations for the test seed phrase,
# spread across addresses 0 & 1:
#
#   Account  Amount
#   0        100gm
#   0        501000test_usd
#   0        1cube
#   0        2000penumbra
#   1        1000test_usd
#   1        1000penumbra
while read -r a; do
    cat <<EOM >> base_allocations.csv
100,gm,$a
5001,test_usd,$a
1,cube,$a
2_000__000_000,upenumbra,$a
EOM
done < <(cut -d' ' -f1 "test_address_0.txt")

while read -r a; do
    cat <<EOM >> base_allocations.csv
1_000,test_usd,$a
1_000__000_000,upenumbra,$a
EOM
done < <(cut -d' ' -f1 "test_address_1.txt")


# Miscellaneous "small" accounts, with just a bit of staking token to pay fees.
# Useful for e.g. bootstrapping relayers on testnets/devnets.
while read -r a; do
    cat <<EOM >> base_allocations.csv
200__000_000,upenumbra,$a
EOM
done < <(cut -d' ' -f1 "small_addresses.txt")

# Copy new base allocations file to target testnet dir.
cp -v base_allocations.csv "$NEW_TESTNET_DIRECTORY/allocations.csv"

echo "All done!"
