# Sending Transactions

Now, for the fun part: sending transactions. If you have someone else's testnet address, you can
send them any amount of any asset you have. 

First, use balance to find the amount of assets you have.
```
cargo run --release --bin pcli view balance
```

Second, if I wanted to send 10 penumbra tokens
to my friend, I could do that like this (filling in their full address at the end):

```bash
cargo run --quiet --release --bin pcli tx send 10penumbra --to penumbrav1t...
```

Notice that asset amounts are typed amounts, specified without a space between the amount (`10`)
and the asset name (`penumbra`).

If you have the asset in your wallet to send, then so it shall be done!

# Staking

In addition, to sending asset, one may also stake penumbra tokens to validators.

Find a validator to stake to:
```
cargo run --release --bin pcli view validator list
```

Copy and paste the address of one of the validators to stake to, then construct the staking tx:

```
cargo run --release --bin pcli tx delegate 10penumbra --to penumbrav1t...
```



