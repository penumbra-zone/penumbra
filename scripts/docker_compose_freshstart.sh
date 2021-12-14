#!/usr/bin/env bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

cd "${parent_path}/../"

die () {
    echo >&2 "$@"
    exit 1
}

[ "$#" -eq 1 ] || die "build path argument required"


build_path="$1"
if [ -d "${build_path}" ] 
then
    printf "Directory ${build_path} already exists. Please remove it if you really want to DELETE ALL YOUR VALIDATOR DATA AND START OVER." >&2
    printf "\n\nIf you just want to rebuild the Docker containers to reflect code changes and maintain the tendermint and penumbra database state, try:\n" >&2
    printf "\t\`docker-compose up --build -d\`" >&2
    exit 1
fi

printf "\t\tA new genesis has occurred...\n\n"
printf "Storing configs to ${build_path}/ ...\n\n\n"

mkdir -p ${build_path}
docker-compose stop
docker container prune
docker volume rm penumbra_tendermint_data
docker volume rm penumbra_prometheus_data
docker volume rm penumbra_db_data
python3 -m venv scripts/.venv
source scripts/.venv/bin/activate
pip install -r scripts/requirements.txt
python3 scripts/setup_validator.py penumbra-valetudo ${build_path}

docker-compose up --build -d
