# Using `pcli` with `pviewd`

First, export a viewing key from `pcli`:
```
pcli wallet export-fvk
```
Next, use the FVK it prints to initialize the `pviewd` state:
```
pviewd init FVK_STRING
```
The location of the `pviewd` state can be changed with the `-s` parameter.
Finally, run
```
pviewd start
```
to start the view server, and invoke `pcli` with
```
pcli -v 127.0.0.1:8081
```
to use it instead of an in-process view service.

**WARNING: the view service does not currently use transport encryption, so it should
not be used over a public network.**
