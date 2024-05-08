# Threshold Custody Backend

This backend allows splitting the spend authority (the ability to *spend* funds) among multiple parties.
Each of these parties will have full viewing authority (the ability to decrypt and view the wallet's on-chain activity).

Threshold custody involves a certain number of parties, `N`, which each have a share, and a threshold `T`, denoting
the number of parties required to sign.
This is often referred to as an `(N, T)` threshold setup.

For example, a `(3, 2)` threshold setup would involve 3 people holding shares, with 2 of them required to sign.

At a high-level, using this backend involves:
1. **(only once)** generating the split keys, either in a centralized or a decentralized manner,
2. **(many times)** signing transactions by having the parties coordinate over a shared secure communication channel.  

For signing, the parties will need to exchange messages to coordinate a threshold signature, and will need
some kind of secure channel to do that; for example, a Signal group.
This channel should:
- provide authenticity to each party sending messages in it,
- encrypt messages, preventing information about on-chain activity from leaking outside the signing parties.

This backend is only accessible from the command-line interface, `pcli`.

## Key-Generation: Centralized

```
pcli init threshold deal --threshold <T> --home <HOME1> --home <HOME2> ...
```

This command will generate the key shares, and then write appropriate pcli configs to `<HOME1>/config.toml`, `<HOME2>/config.toml`, etc.
The number of parties is controlled only by the number of home directories passed to this command.
The threshold required to sign will be `<T>`.


### Security

The computer running this command will have access to all the shares at the moment of generation.
This can be useful to centrally provision a threshold setup, but should be done on a secure computer
which gets erased after the setup has been performed.

## Key-Generation: Decentralized

```
pcli init threshold dkg --threshold <T> --num-participants <N>
```

This command will generate the key shares in a decentralized manner.
Each party will run this command on the machine where they want their share to be.
The command will prompt them to communicate information with the other parties,
and to relay the information they've received back into it, before eventually
producing a key share, and writing it to the default home directory used by `pcli`.

This method is more secure, because no computer ever has full access to all shares.

## Signing

For signing, one party must coordinate signing, and the other parties
will follow along and review the suggested transaction.

The coordinator will use `pcli` as they would with the standard single-party custody backend.
When it comes time for `pcli` to produce a signature, the command will then prompt the user
to communicate information with the other parties, and relay their response back, in order
to produce a signature.

The followers run a separate command to participate in signing:
```
pcli threshold sign
```
This will have them be prompted to input the coordinator's information, and will display
a summary of the transaction, which they should independently review to check that it's
actually something they wish to sign.

Note that only a threshold number of participants are required to sign, and any others do not
need to participate. So, for example, with a threshold of `2`, only one other follower beyond
the coordinator is needed, and additional followers *cannot* be used.

## Communication Channel

The `dkg` and `sign` commands will spit out blobs of information that need to be relayed between
the participants securely.
An end-to-end example of how this process works is captured in this video:
[https://twitter.com/penumbrazone/status/1732844637180862603](https://twitter.com/penumbrazone/status/1732844637180862603)

## Encryption

A password can be used to generate an encrypted config via:
```bash
$ pcli init --encrypted threshold dkg ...
```

Furthermore, an existing config can be converted to an encrypted one with:
```bash
$ pcli init re-encrypt
```
