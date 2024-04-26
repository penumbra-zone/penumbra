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