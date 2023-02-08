#!/usr/bin/env bash
set -eu

# change to subdir where compose files are stored
compose_dir="$(git rev-parse --show-toplevel)/deployments/compose"
cd "$compose_dir" || exit 1

build_path="$HOME/.penumbra/testnet_data/"

if [ -d "${build_path}" ] 
then
    printf "Directory %s already exists. Please remove it if you really want to DELETE ALL YOUR VALIDATOR DATA AND START OVER." "$build_path" >&2
    printf "\n\nIf you just want to rebuild the Docker containers to reflect code changes and maintain the tendermint and penumbra database state, try:\n" >&2
    printf "\t\`cd deployments/compose && docker-compose up --build -d\`" >&2
    exit 1
fi

printf "\t\tA new genesis has occurred...\n\n"
printf "Storing configs to %s/ ...\n\n\n" "$build_path"

docker-compose stop
docker container prune -f
docker volume rm penumbra_prometheus_data || true

cargo run --release --bin pd -- testnet --testnet-dir "${build_path}" generate --epoch-duration 10
