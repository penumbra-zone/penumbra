# Penumbra deployments

This directory contains config management logic for managing
Penumbra networks. As of 2022Q4, prior to mainnet,
Penumbra Labs runs two (2) discrete networks:

  * "testnet", updated approximately weekly
  * "testnet-preview", updated on every push to `main` in the repo

Those networks each have their own genesis and knowledge of peers.
The networks are completely separate.

## Directory structure

```
.
├── ci.sh # runner script for executing a deploy against k8s
├── helm/ # helm chart used to configure full-node/validator layout
├── networks/ # logic specific to network, e.g. "testnet" or "testnet-preview"
│  └── testnet/
└── terraform/ # server and cluster provisioning logic
   └── modules/
```

## Out of band config
There are several DNS records that are not handled
by the automation in this repo. Each testnet should have:

* `fullnode.<fqdn>` # suitable for `pd testnet join <node>`
* `rpc.<fqdn>` # the tendermint rpc port
* `grpc.<fqdn>` # the pd grpc port

To find the IPv4 address for `fullnode.<fqdn>`, run:

```
# N.B. the string "penumbra-testnet" maps to $HELM_RELEASE in the ci.sh script.
kubectl get svc penumbra-testnet-p2p-fn-0 --output jsonpath='{.status.loadBalancer.ingress[0].ip}'
```

To find the IPv4 address for `{g,}rpc.<fqdn>`, hop into the relevant
network directory, then check Terraform outputs:

```
cd networks/testnet
terraform output
```

The reserved IPv4 address will be displayed.
