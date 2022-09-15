#!/bin/bash
WORKDIR=${WORKDIR:=$(pwd)/pdcli}
IMAGE=${IMAGE:=ghcr.io/strangelove-ventures/heighliner/penumbra}
PENUMBRA_VERSION=${PENUMBRA_VERSION:=026-hermippe}
TENDERMINT_VERSION=${TENDERMINT_VERSION:=v0.35.9}
NVALS=${NVALS:=2}
NFULLNODES=${NFULLNODES:=1}
CONTAINERHOME=${CONTAINERHOME:=/root}
HELM_RELEASE=${HELM_RELEASE:=testnet}

# Use fresh working directory
rm -rf ${WORKDIR}
mkdir -p "${WORKDIR}"

echo "Shutting down existing testnet if necessary..."
# Delete existing replication controllers
kubectl delete rc --all --wait=false 2>&1 > /dev/null
# Delete all existing PVCs so that fresh testnet is created
kubectl delete pvc --all 2>&1 > /dev/null

for i in $(seq $NVALS)
do
    I="$((i-1))"
    NODEDIR="node$I"
    echo "node$I - generating keys"
    mkdir -p "${WORKDIR}/$NODEDIR"
    docker run --user 0:0 \
    -v "$WORKDIR/$NODEDIR":"$CONTAINERHOME" -it --rm \
    --entrypoint pcli \
    $IMAGE:$PENUMBRA_VERSION \
    -d "$CONTAINERHOME" keys generate > /dev/null

    echo "node$I - generating validator template definition"
    docker run --user 0:0 \
    -v "$WORKDIR/$NODEDIR":"$CONTAINERHOME" -it --rm \
    --entrypoint pcli \
    $IMAGE:$PENUMBRA_VERSION \
    -d "$CONTAINERHOME" validator definition template \
    --file "$CONTAINERHOME"/val.json > /dev/null
done

find "$WORKDIR" -name "val.json" -exec cat {} + | jq -s > "$WORKDIR/vals.json"

echo "Generating new testnet files..."
docker run --user 0:0 \
-v "$WORKDIR":"$CONTAINERHOME" -it --rm \
--entrypoint pd \
$IMAGE:$PENUMBRA_VERSION \
testnet generate \
--validators-input-file "$CONTAINERHOME/vals.json" > /dev/null

sudo chown -R "$(whoami)" "$WORKDIR"

for i in $(seq $NVALS)
do
    I=$((i-1))
    NODE_ID=$(jq -r '.priv_key.value' ./pdcli/.penumbra/testnet_data/node$I/tendermint/config/node_key.json | base64 --decode | tail -c 32 | sha256sum  | cut -c -40)
    if [ "$I" -eq "0" ]; then
      PERSISTENT_PEERS=$NODE_ID@p2p-$I
    else
      PERSISTENT_PEERS="$PERSISTENT_PEERS,$NODE_ID@p2p-$I"
    fi
done

echo "$PERSISTENT_PEERS" > persistent_peers.txt

helm get values $HELM_RELEASE
if [ "$?" -eq "0" ]; then
  HELM_CMD=upgrade
else
  HELM_CMD=install
fi

helm $HELM_CMD $HELM_RELEASE . --set count=$NVALS --set penumbraVersion=$PENUMBRA_VERSION --set tendermintVersion=$TENDERMINT_VERSION