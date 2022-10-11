#!/usr/bin/env bash
set -eu

parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

cd "${parent_path}/../"

die () {
    echo >&2 "$@"
    exit 1
}

build_path="$HOME/.penumbra/testnet_data/"

if [ -d "${build_path}" ] 
then
    printf "Directory ${build_path} already exists. Please remove it if you really want to DELETE ALL YOUR VALIDATOR DATA AND START OVER." >&2
    printf "\n\nIf you just want to rebuild the Docker containers to reflect code changes and maintain the tendermint and penumbra database state, try:\n" >&2
    printf "\t\`docker-compose up --build -d\`" >&2
    exit 1
fi

printf "\t\tA new genesis has occurred...\n\n"
printf "Storing configs to ${build_path}/ ...\n\n\n"

docker-compose stop
docker container prune -f
docker volume rm penumbra_prometheus_data || true

cargo run --release --bin pd -- testnet --testnet-dir ${build_path} generate --epoch-duration 10
