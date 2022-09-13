#!/bin/bash
WORKDIR=${WORKDIR:=$(pwd)/pdcli}
IMAGE=${IMAGE:=ghcr.io/strangelove-ventures/heighliner/penumbra}
VERSION=${VERSION:=026-hermippe}
NVALS=${NVALS:=2}
NFULLNODES=${NFULLNODES:=1}
CONTAINERHOME=${CONTAINERHOME:=/root}
mkdir -p "${WORKDIR}"

for i in $(seq $NVALS)
do
    NODEDIR="node$((i-1))"
    mkdir -p "${WORKDIR}/$NODEDIR"
    docker run --user 0:0 \
    -v "$WORKDIR/$NODEDIR":"$CONTAINERHOME" -it --rm \
    --entrypoint pcli \
    $IMAGE:$VERSION \
    -d "$CONTAINERHOME" keys generate

    docker run --user 0:0 \
    -v "$WORKDIR/$NODEDIR":"$CONTAINERHOME" -it --rm \
    --entrypoint pcli \
    $IMAGE:$VERSION \
    -d "$CONTAINERHOME" validator definition template \
    --file "$CONTAINERHOME"/val.json
done

find "$WORKDIR" -name "val.json" -exec cat {} + | jq -s > "$WORKDIR/vals.json"


docker run --user 0:0 \
-v "$WORKDIR":"$CONTAINERHOME" -it --rm \
--entrypoint pd \
$IMAGE:$VERSION \
testnet generate \
--validators-input-file "$CONTAINERHOME/vals.json"


helm install testnet . --set count=$NVALS

# kubectl create configmap node-config --from-file=config/ -o yaml --dry-run=client | kubectl apply -f -
