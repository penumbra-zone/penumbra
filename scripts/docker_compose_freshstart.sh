#!/usr/bin/env bash

docker-compose stop
docker container prune
docker volume rm penumbra_tendermint_data
docker volume rm penumbra_prometheus_data
docker volume rm penumbra_db_data
docker-compose up --build -d
python3 -m venv scripts/.venv
source scripts/.venv/bin/activate
pip install -r scripts/requirements.txt
python3 scripts/setup_validator.py testnets/001_valetudo.csv
