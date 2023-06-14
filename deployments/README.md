# Penumbra deployments

This directory contains config management logic for managing
Penumbra networks. As of 2023Q1, prior to mainnet,
Penumbra Labs runs three (3) discrete networks:

  * "testnet", updated approximately weekly
  * "testnet-preview", updated on every push to `main` in the repo
  * "devnet", updated ad-hoc to serve as a sandbox debugging environment

Those networks each have their own genesis and knowledge of peers.
The networks are completely separate.

## Directory structure

```
.
├── ci.sh # runner script for executing a deploy against k8s
├── charts/ # helm charts used to configure full-node/validator layout
├── networks/ # logic specific to network, e.g. "testnet" or "testnet-preview"
│  └── testnet/
└── terraform/ # server and cluster provisioning logic
   └── modules/
```

## Running custom devnets

Sometimes you want to perform a deploy that mimics our preview and testnet setups,
but has its own lifecycle. For example, you may want to test new cluster config logic,
or try to reproduce a bug on an older version of the software. Behold:

```
cd deployments/
export HELM_RELEASE=penumbra-devnet
./ci.sh
```

That will deploy a new network based on latest `main` (containers are rebuilt on every merge)
and run it. You can destroy it when you're done with `helm uninstall penumbra-devnet`.
You can also run an older version of the software:

```
cd deployments/
export HELM_RELEASE=penumbra-devnet
export PENUMBRA_VERSION=v0.53.1
./ci.sh
```

You'll see a message about a "patch release"; you'll need to edit the ci script's main function to force
a run of the full deploy logic.

## Out of band config
There are several DNS records that are not handled
by the automation in this repo. Each testnet should have:

* `rpc.<fqdn>` # the tendermint rpc service
* `grpc.<fqdn>` # the pd grpc service
* `grafana.<fqdn>` # web interface for metrics dashboards

To find the IPv4 address for `{g,}rpc.<fqdn>`, use this command:

```
kubectl get svc -n infra traefik --output jsonpath='{.status.loadBalancer.ingress[0].ip}'
```

The Traefik reverse proxy is used for fronting pd's grpc service, because Traefik supports h2c.
See for details: https://github.com/penumbra-zone/penumbra/issues/2341

## Dude, where's my logs?

There's web-based access for viewing logs from the testnet deployment:

* [Top-level view of all deployments](https://console.cloud.google.com/kubernetes/workload/overview?project=penumbra-sl-testnet)
* [Logs for the deployment with RPC endpoints exposed](https://console.cloud.google.com/kubernetes/deployment/us-central1/testnet/default/penumbra-testnet-fn-0/logs?project=penumbra-sl-testnet)

You must authenticate with your PL Google account to view that information;
ask a team member if you need a grant added for your account.
