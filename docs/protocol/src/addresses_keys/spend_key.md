# Spending Keys

A [BIP39] 12- or 24-word seed phrase can be used to derive one or more spend
authorities. 

## Legacy Raw BIP39 Derivation

Prior to Testnet 62, from this mnemonic seed phrase, spend `seeds` were derived
using PBKDF2 with:

* `HMAC-SHA512` as the PRF and an iteration count of 2048 (following [BIP39])
* the seed phrase used as the password
* `mnemonic` concatenated with a passphrase used as the salt, where the default
spend authority was derived using the salt `mnemonic0`.

## Default BIP44 Derivation

Beginning in Testnet 62, from the mnemonic seed phrase, spend `seeds` were derived
as described in [BIP44]. The BIP44 specification describes a organizational
hierarchy allowing a user to remember a single seed phrase for multiple
cryptocurrencies. 

The BIP44 path for Penumbra consists of:

```
m / purpose' / coin_type' / wallet_id'
```

`m` represents the master node and is derived from the spend seed as described in
[BIP32] in section "Master key generation".

The purpose field is a constant set to `44'` to denote that BIP44 is being used.

Penumbra's registered `coin_type` is defined in [SLIP-0044]:

* Coin type: `6532`
* Path component `coin_type' = 0x80001984`

The default wallet ID is set to 0. A typical use case for Penumbra will involve
generating the single default wallet, and then using multiple Penumbra accounts
within that wallet which share a single viewing key.

The BIP44 path is used with the seed phrase to derive the spend `seed` for use
in Penumbra following the child key derivation specified in [BIP32].

The root key material for a particular spend authority is the 32-byte
`spend_key_bytes` derived as above from the seed phrase. The `spend_key_bytes` value is used to derive

* $\mathsf{ask} \in \mathbb F_r$, the *spend authorization key*, and
* $\mathsf{nk} \in \mathbb F_q$, the *nullifier key*,

as follows.  Define `prf_expand(label, key, input)` as BLAKE2b-512 with
personalization `label`, key `key`, and input `input`.  Define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order.  Then
```
ask = from_le_bytes(prf_expand("Penumbra_ExpndSd", spend_key_bytes, 0)) mod r
nk  = from_le_bytes(prf_expand("Penumbra_ExpndSd", spend_key_bytes, 1)) mod q
```

The *spending key* consists of `spend_key_bytes` and `ask`.  (Since `ask` is
derived from `spend_key_bytes`, only the `spend_key_bytes` need to be stored,
but the `ask` is considered part of the spending key). When using a hardware
wallet or similar custody mechanism, the spending key remains on the device.

The spend authorization key $\mathsf{ask}$ is used as a `decaf377-rdsa` signing
key.[^1] The corresponding verification key is the *spend verification key*
$\mathsf{ak} = [\mathsf{ask}]B$.  The spend verification key $\mathsf{ak}$ and
the nullifier key $\mathsf{nk}$ are used to create the *full viewing key*
described in the next section.

[^1]: Note that it is technically possible for the derived $ask$ or $nsk$ to be
$0$, but this happens with probability approximately $2^{-252}$, so we ignore
this case, as, borrowing phrasing from [Adam Langley][agl_elligator], it happens
significantly less often than malfunctions in the CPU instructions we'd use to
check it.

[agl_elligator]: https://www.imperialviolet.org/2013/12/25/elligator.html
[BIP32]: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
[BIP39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
[BIP44]: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
[SLIP-0044]: https://github.com/satoshilabs/slips/blob/master/slip-0044.md