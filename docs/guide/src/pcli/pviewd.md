# Using `pcli` with `pviewd`

First, build `pviewd` binnary:

```shell
cargo build --release --bin pviewd
```

Then you either can modify your `$PATH` variable:

```shell
export $PATH=$PATH:$HOME/penumbra/target/release
```
or copy the binnary file to `/usr/bin` directory:

```shell
cp /root/penumbra/target/release/pviewd /usr/bin/
```
after, you need to export a viewing key from `pcli`:

```shell
pcli keys export full-viewing-key
```

Next, use the FVK it prints to initialize the `pviewd` state:

```shell
pviewd init <YOUR_FULL_VIEWING_KEY>
```

The location of the `pviewd` state can be changed with the `-s` parameter.
Finally, run

```shell
pviewd start
```

to start the view server, and invoke `pcli` with

```shell
pcli -v 127.0.0.1:8081
```

to use it instead of an in-process view service.

**WARNING: the view service does not currently use transport encryption, so it should
not be used over a public network.**
