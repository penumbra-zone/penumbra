#!/bin/bash
# The purpose of this script is to replicate the failure conditions in GH1884.
# It attempts to restart tendermint fast enough after first start that
# the InitChain request gets resent on subsequent service start.
set -euo pipefail
set -x


sudo systemctl disable --now penumbra tendermint

# build new binaries
cargo build --release
pd testnet unsafe-reset-all
pd testnet join testnet-preview.penumbra.zone

sudo systemctl restart penumbra tendermint
sleep 2s
sudo systemctl stop tendermint
sleep 5s
sudo systemctl restart tendermint
sudo journalctl -af -u tendermint
