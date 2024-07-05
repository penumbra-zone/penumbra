# Running a fullnode

In order to interact with the Penumbra network, users must provide an RPC URL
so that client software can read chain state and submit transactions.
This is true of the [Prax wallet], and of [pcli], as well as any other client.
While users can select a publicly available RPC URL from a party they trust,
this guide demonstrates how a user can self-host an RPC URL for use by themselves and others.
For a more generalized description of running pd, see the [pd overview](../node/pd.md).

## Renting a server

There are a variety of cloud providers that provide dedicated hardware for a per-month cost basis.
Generally, hardware-based solutions will have superior performance, particularly in storage latency,
and also more reliable performance over time. One suitable option is the
[Matrix AX52 by Hetzner](https://www.hetzner.com/dedicated-rootserver/ax52/).

To get started with Hetzner, [create an account](https://accounts.hetzner.com/signUp), provide billing information,
then request a dedicated hardware server. While preparing the server request,
you'll need to provide an SSH public key for the root user account. You can use this command to generate one
if you don't have one already:

```
ssh-keygen -t ed25519
cat ~/.ssh/id_ed25519.pub
```

Choose the Debian Stable option for operating system.
Shortly after requesting the server, you should receive an email notifying you that it's ready to accept logins.

## Setting up DNS

In order to use HTTPS over the web interface, you'll need to create an A record for the domain you want to use,
pointing to the IPv4 address for the server. Visit the website for your DNS provider, and create the A record,
using the IP address displayed on the server page for Hetzner.

## Provisioning the server

Log into the server like so:

```
ssh -l root <YOUR_DNS_DOMAIN>
```

If that command fails, you'll need to debug your access settings.

First, clone the git repository:

```
apt-get install -y git git-lfs
git clone --branch {{ #include ../penumbra_version.md }} https://github.com/penumbra-zone/penumbra
```

Use that repo to copy the service configurations into place:

```
cd penumbra/deployments/systemd/
cp penumbra.service cometbft.service /etc/systemd/system/
# edit /etc/systemd/system/penumbra.service,
# and add your DNS domain to the `--grpc-auto-https` example.
systemctl daemon-reload
```

Follow the guide to [install pd and cometbft](../node/pd/install.md) from their respective
release pages. You should be able to run `pd --version` and see `{{ #include ../penumbra_version.md }}`
displayed. 

Next, create a user account for running the Penumbra software:

```
sudo useradd -m -d /home/penumbra penumbra -s /bin/bash
```

We'll use this account to configure the `pd` and `cometbft` data directories.

```
sudo su -l penumbra
pd network join \
       --moniker <MONIKER> \
       --external-address <EXTERNAL_ADDRESS> \
       <NODE_URL>
```

The value for `NODE_URL` should be the CometBFT RPC endpoint for the node whose network
you want to join. Change `MONIKER` to the human-readable name for your node on the network.
Finally, the `EXTERNAL_ADDRESS` should be the public IP address of the server, so that
other peers on the network can initiate connections to it, to share blocks.

## Running the node
Finally, start the services:

```
# return to root user
exit
systemctl restart penumbra cometbft
journalctl -af -u penumbra
```

The final command will display logs from the `pd` process. In a short while, you should see
blocks streaming in. If not, see the [debugging steps](../node/pd/debugging.md)
to figure out what went wrong.

[pcli]: ../pcli.md
[Prax wallet]: https://chromewebstore.google.com/detail/prax-wallet/lkpmkhpnhknhmibgnmmhdhgdilepfghe
