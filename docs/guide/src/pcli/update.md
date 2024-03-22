## Updating `pcli`

Follow the [installation steps](install.md) to install the 
most recent version of `pcli`, which is `{{ #include ../penumbra_version.md }}`.

After installing the updated version, reset the view data used by `pcli`:

```
pcli view reset
```

No wallet needs to be [generated](wallet.md#generating-a-wallet). The existing wallet
will be used automatically.
