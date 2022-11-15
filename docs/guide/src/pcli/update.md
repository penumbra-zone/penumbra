## Updating `pcli`

Follow the [same steps](https://guide.penumbra.zone/main/pcli/install.html#cloning-the-repository) to update to the latest testnet [release](https://github.com/penumbra-zone/penumbra/releases)

```
cd penumbra && git fetch && git checkout 035-taygete && cargo update
```

Once again, build `pcli` with cargo

```
cargo build --release --bin pcli
```

No wallet needs to be [generated](https://guide.penumbra.zone/main/pcli/wallet.html#generating-a-wallet). Instead, keep one's existing wallet and reset view data.

```
cargo run --quiet --release --bin pcli view reset
```
