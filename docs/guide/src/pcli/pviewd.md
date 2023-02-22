# Using `pcli` with `pclientd`

First, export a viewing key from `pcli`:

```shell
pcli keys export full-viewing-key
```

Next, use the FVK it prints to initialize the `pclientd` state:

```shell
pclientd init FVK_STRING
```

The location of the `pclientd` state can be changed with the `-s` parameter.
Finally, run

```shell
pclientd start
```

to start the view server, and invoke `pcli` with

```shell
pcli -v 127.0.0.1:8081
```

to use it instead of an in-process view service.

**WARNING: the view service does not currently use transport encryption, so it should
not be used over a public network.**
