# Software Custody Backend

The `softkms` custody backend stores the spending key unencrypted in the pcli configuration file.

To generate a new wallet, try:
```bash
$ pcli init soft-kms generate
YOUR PRIVATE SEED PHRASE:
[SEED PHRASE]
Save this in a safe place!
DO NOT SHARE WITH ANYONE!
Writing generated config to [PATH TO PCLI DATA]
```

Alternatively, to import an existing wallet, try
```bash
$ pcli init soft-kms import-phrase
Enter seed phrase:
Writing generated config to [PATH TO PCLI DATA]
```

## Encryption

A password can be used to generate an encrypted config via:
```bash
$ pcli init --encrypted soft-kms ...
```
with either the `generate`, or the `import-phrase` command.

Furthermore, an existing config can be converted to an encrypted one with:
```bash
$ pcli init re-encrypt
```

