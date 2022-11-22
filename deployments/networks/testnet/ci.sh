#!/bin/bash
WORKDIR=${WORKDIR:=$(pwd)/helm/pdcli}
IMAGE=${IMAGE:=ghcr.io/strangelove-ventures/heighliner/penumbra}
PENUMBRA_VERSION=${PENUMBRA_VERSION:=030-isonoe}
PENUMBRA_UID_GID=${PENUMBRA_UID_GID:=1025\:1025}
TENDERMINT_VERSION=${TENDERMINT_VERSION:=v0.34.23}
NVALS=${NVALS:=2}
NFULLNODES=${NFULLNODES:=1}
CONTAINERHOME=${CONTAINERHOME:=/root}
HELM_RELEASE=${HELM_RELEASE:=testnet}

# Use fresh working directory
sudo rm -rf ${WORKDIR}
mkdir -p "${WORKDIR}"

echo "Shutting down existing testnet if necessary..."
# Delete existing replication controllers
kubectl delete rc --all --wait=false 2>&1 > /dev/null
# Delete all existing PVCs so that fresh testnet is created
kubectl delete pvc --all 2>&1 > /dev/null

for i in $(seq $NVALS); do
    I="$((i-1))"
    NODEDIR="node$I"
    mkdir -p "${WORKDIR}/$NODEDIR"
    # This will be overwritten by pd testnet generate.
    echo '{"identity_key": "penumbravalid1lr73zgd726gpk7rl45hvpg9f7r9wchgg8gpjhx2gqntx4md6gg9sser05u","consensus_key": "9OQ8HOy4YsryEPLbTtPKoKdmmjSqEJhzvS+x0WC8YoM=","name": "","website": "","description": "","enabled": false,"funding_streams": [{"address": "penumbrav2t1wz70yfqlgzfgwml5ne04vhnhahg8axmaupuv7x0gpuzesfhhz63y52cqffv93k7qvuuq6yqtgcj0z267v59qxpjuvc0hvfaynaaemgmqzyj38xhj8yjx7vcftnyq9q28exjrdj","rate_bps": 100}],"sequence_number": 0,"governance_key": "penumbragovern1lr73zgd726gpk7rl45hvpg9f7r9wchgg8gpjhx2gqntx4md6gg9sthagp6"}' > "${WORKDIR}/$NODEDIR/val.json"
done

find "$WORKDIR" -name "val.json" -exec cat {} + | jq -s > "$WORKDIR/vals.json"

echo "Generating new testnet files..."
docker run --user 0:0 \
-v "$WORKDIR":"$CONTAINERHOME" --rm \
--entrypoint pd \
$IMAGE:$PENUMBRA_VERSION \
testnet generate \
--validators-input-file "$CONTAINERHOME/vals.json" > /dev/null

sudo chown -R "$(whoami)" "$WORKDIR"

for i in $(seq $NVALS); do
    I=$((i-1))
    NODE_ID=$(jq -r '.priv_key.value' ${WORKDIR}/.penumbra/testnet_data/node$I/tendermint/config/node_key.json | base64 --decode | tail -c 32 | sha256sum  | cut -c -40)
    for j in $(seq $NVALS)
    do
      J=$((j-1))
      if [ "$I" -ne "$J" ]; then
        PVAR=PERSISTENT_PEERS_$J
        if [ -z "${!PVAR}" ]; then
          declare PERSISTENT_PEERS_$J="$NODE_ID@p2p-$I:26656"
        else
          declare PERSISTENT_PEERS_$J="$PERSISTENT_PEERS,$NODE_ID@p2p-$I:26656"
        fi
      fi
    done
    if [ -z "$PERSISTENT_PEERS" ]; then
      PERSISTENT_PEERS="$NODE_ID@p2p-$I:26656"
      PRIVATE_PEERS="$NODE_ID"
    else
      PERSISTENT_PEERS="$PERSISTENT_PEERS,$NODE_ID@p2p-$I:26656"
      PRIVATE_PEERS="$PRIVATE_PEERS,$NODE_ID"
    fi
done

for i in $(seq $NVALS); do
  I=$((i-1))
  PVAR=PERSISTENT_PEERS_$I
  echo "${!PVAR}" > $WORKDIR/persistent_peers_$I.txt
done

echo "$PERSISTENT_PEERS" > $WORKDIR/persistent_peers.txt
echo "$PRIVATE_PEERS" > $WORKDIR/private_peers.txt

helm get values $HELM_RELEASE 2>&1 > /dev/null
if [ "$?" -eq "0" ]; then
  HELM_CMD=upgrade
else
  HELM_CMD=install
fi

echo "Deploying network..."

helm $HELM_CMD $HELM_RELEASE helm --set numValidators=$NVALS,numFullNodes=$NFULLNODES,penumbra.image=$IMAGE,penumbra.version=$PENUMBRA_VERSION,penumbra.uidGid=$PENUMBRA_UID_GID,tendermint.version=$TENDERMINT_VERSION

while true; do
  echo "Waiting for load balancer external IPs to be provisioned..."
  STATUSES=($(kubectl get svc --no-headers | grep p2p | awk '{print $4}'))
  FOUND_PENDING=false
  for STATUS in "${STATUSES[@]}"; do
    if [[ "$STATUS" == "<pending>" ]]; then
      sleep 5
      FOUND_PENDING=true
      break
    fi
  done
  if [[ "$FOUND_PENDING" == "false" ]]; then
    break
  fi
done

RETRIES=0
while true; do
  echo "Waiting for pods to be running..."
  PODS=($(kubectl get pods --no-headers | awk '{print $1}'))
  FOUND_PENDING=false
  for POD in "${PODS[@]}"; do
    STATUS=$(kubectl get pod --no-headers "$POD" | awk '{print $3}')
    if [[ "$STATUS" == "Error" ]] || [[ "$STATUS" == "CrashLoopBackOff" ]]; then
      echo "Node $POD startup failed!"
      kubectl logs $POD
      exit 1
    fi
    if [[ "$STATUS" != "Running" ]]; then
      RETRIES=$((RETRIES+1))
      if [[ "$RETRIES" == "50" ]]; then
        echo "Giving up starting nodes"
        exit 1
      fi
      sleep 5
      FOUND_PENDING=true
      break
    fi
  done
  if [[ "$FOUND_PENDING" == "false" ]]; then
    break
  fi
done

PPE=""

for i in $(seq $NVALS); do
  I=$((i-1))
  echo "Getting public peer string for validator $I"
  NODE_ID="$(kubectl exec $(kubectl get pods | grep penumbra-val-$I | awk '{print $1}') -c tm -- tendermint --home=/home/.tendermint show-node-id | tr -d '\r')"
  IP="$(kubectl get svc p2p-$I -o json | jq -r .status.loadBalancer.ingress[0].ip | tr -d '\r')"
  if [ -z "$PPE" ]; then
    PPE="$NODE_ID@$IP:26656"
  else
    PPE="$PPE,$NODE_ID@$IP:26656"
  fi
done

for i in $(seq $NFULLNODES); do
  I=$((i-1))
  echo "Getting public peer string for fullnode $I"
  NODE_ID="$(kubectl exec $(kubectl get pods | grep penumbra-fn-$I | awk '{print $1}') -c tm -- tendermint --home=/home/.tendermint show-node-id | tr -d '\r')"
  IP="$(kubectl get svc p2p-fn-$I -o json | jq -r .status.loadBalancer.ingress[0].ip | tr -d '\r')"
  PPE="$PPE,$NODE_ID@$IP:26656"
done

echo "persistent_peers = \"$PPE\""

