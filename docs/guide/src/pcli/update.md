## Updating `pcli`

Make sure you've followed the [installation steps](https://guide.penumbra.zone/main/pcli/install.html#cloning-the-repository). Then, to update to the latest testnet [release](https://github.com/penumbra-zone/penumbra/releases):

```
cd penumbra && git fetch && git checkout 044-ananke
```

Once again, build `pcli` with cargo:

```
cargo build --release
```

No wallet needs to be [generated](https://guide.penumbra.zone/main/pcli/wallet.html#generating-a-wallet). Instead, keep one's existing wallet and reset view data.


```
pcli view reset
```

If you see an error containing `pcli: command not found`, make sure you have created a symlink for `pcli`,
as described in the [install guide](https://guide.penumbra.zone/main/pcli/install.html).
