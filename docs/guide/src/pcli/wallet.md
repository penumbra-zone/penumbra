# Generating a Wallet

On first [installation of `pcli`](./install.md), you will need to generate a fresh wallet to use with Penumbra.

The `pcli init` command will generate a configuration file, depending on the
custody backend used to store keys.

There are currently three custody backends:

1. The [`softkms` backend](./wallet/softkms.md) is a good default choice for low-security use cases.  It stores keys unencrypted in a local config file.
2. The [`threshold` backend](./wallet/threshold.md) is a good choice for high-security use cases. It provides a shielded multisig, with key material sharded over multiple computers.
3. The `view-only` backend has no custody at all and only has access to viewing keys.

Furthermore, `softkms` and `threshold` allow encrypting the spend-key related material with a password.

After running `pcli init` with one of the backends described above, `pcli` will be initialized.

## Shielded accounts

Penumbra's design automatically creates `2^32` (four billion) numbered accounts
controlled by your wallet.

To generate the address for a numbered account, use `pcli view address`:

```bash
$ pcli view address 0
penumbrav2t1...
```

You can also run `pcli view address` on an address to see which account it corresponds to:

```bash
$ pcli view address penumbra1...
Address is viewable with this full viewing key. Account index is 0.
```

Addresses are opaque and do not reveal account information. Only you, or someone
who has your viewing key, can decrypt the account information from the address.

## Getting testnet tokens on Discord in the `#testnet-faucet` channel

In order to use the testnet, it's first necessary for you to get some testnet tokens. The current
way to do this is to join our [Discord](https://discord.gg/hKvkrqa3zC) and post your address in the `#testnet-faucet` channel.
We'll send your address some tokens on the testnet for you to send to your friends! :)

Just keep in mind: **testnet tokens do not have monetary value**, and in order to keep the
signal-to-noise ratio high on the server, requests for tokens in other channels will be deleted
without response. Please do not DM Penumbra Labs employees asking for testnet tokens; the correct
venue is the dedicated channel.

## Validator custody

Validators need to custody more kinds of key material than ordinary users. A validator operator has
control over:

- The validator's **identity signing key**: the root of the validator's
  identity, controlling its on-chain definition and all other subkeys. 
  The public half of this keypair is contained in a Penumbra validator ID such as
  `penumbravalid1u2z9c75xcc2ny6jxccge6ehqtqkhgy4ltxms3ldappr06ekpguxqq48pdh`.
  This key
  *can never be rotated*, so it is very important for a validator to keep it
  secure.  However, the identity signing key is _only_ used for signing changes to
  the validator's configuration data, so it can be kept in cold storage or via
  threshold (multisig) custody. 
- The validator's **CometBFT consensus key**: the key used to sign blocks by the running validator.
  Unlike the Cosmos SDK, the consensus key can be freely rotated by uploading a
  new validator definition with an updated consensus key.  It is important to
  secure all historical consensus keys used by a validator, as compromise could
  lead to double-signing and slashing.  Secure custody of consensus keys is
  outside the scope of this document. For examples of ways to custody this key
  material, see [Horcrux](https://github.com/strangelove-ventures/horcrux) and
  [tmkms](https://github.com/iqlusioninc/tmkms), two approaches for
  high-security online custody of CometBFT consensus keys.
- The validator's **governance signing key**: the key used to vote on governance
  proposals in the capacity as a validator.  Unlike the Cosmos SDK, this is also
  a subkey, allowing validators to vote on governance proposals without taking
  their most sensitive key material out of cold storage. By default, this is
  identical to the validator's identity key, but it can be manually set to a
  different key. To do this, use the command `pcli init validator-governance-subkey` 
  after you have run `pcli init`. This command
  requires that you choose a custody backend for the governance subkey, which
  can be one of `soft-kms` or `threshold`.  The same options apply to these as
  above.
- The validator's **treasury wallet(s)**: the wallet(s) which hold the validator's self-delegated
  funds, and which receive output from (some of) the validator's funding stream(s). These are
  configured entirely separately from the validator's identity key, and may be changed at any time
  by migrating funds from one wallet to another and/or updating the funding streams in the
  validator's on-chain definition. The default templated definition uses an address derived from the
  validator's identity key wallet, but most operators likely want to change this.

Because Penumbra permits validators to separate their key material as above, validators can choose
different custody solutions for the different risk profiles and use cases of different keys,
independently.

### Multiple signatures when using threshold custody

Validator definitions are self-authenticating data packets, signed by the
validator's identity key.  Updating a validator definition requires two steps:
first, producing a signed validator definition, and second, relaying that
definition onto the chain.  In a low-security setup, `pcli` can do these steps automatically.

However, when a validator is using a threshold custody backend for the identity
key, these steps may require separate signing operations. First, the
*definition* must be signed (using the custody method of the identity key), and
then that definition must be broadcast to the chain.  Any wallet can broadcast
the signed validator definition, similar to the way that any wallet can relay IBC packets.

Similarly, when a validator is using a threshold custody backend for the
governance subkey, casting a vote as a validator may require separate steps,
because the vote is signed independently of the transaction which broadcasts it.

### Airgap or Wormhole custody

If a validator wishes to use an airgapped signing setup (with or without threshold custody) to sign
definition updates or governance votes, this is possible:

- To sign a definition over an airgap, produce a signature on the airgapped machine or machines
  using `pcli validator definition sign`, then upload the definition on a networked machine, after
  copying the signature across the airgap, using `pcli validator definition upload` with the optional
  `--signature` flag to specify the externally-produced signature for the definition.
- To sign a validator vote over an airgap, produce a signature on the airgapped machine or machines
  using `pcli validator vote sign`, then upload the vote on a networked machine, after copying the
  signature across the airgap, using `pcli validator vote cast` with the optional `--signature` flag
  to specify the externally-produced signature for the vote.

Alternatively, rather than using a literal airgap, [magic
wormhole](https://magic-wormhole.readthedocs.io/en/latest/) is a fast and secure
method for relaying data between computers without complex network interactions.
