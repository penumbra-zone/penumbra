# systemd unit files for Penumbra

Here are example unit files for running Penumbra (and the required Tendermint instance)
via systemd. *You'll need to customize the the service files*, particularly the `User` declaration
and binary fullpath in `ExecStart`, and possibly the path to the home directory, depending on your system.

## Installing
Copy the service files to a system-wide location:

```bash
# use 'envsubst' to replace `$USER` with your local username
envsubst < penumbra.service | sudo tee /etc/systemd/system/penumbra.service
envsubst < tendermint.service | sudo tee /etc/systemd/system/tendermint.service
sudo systemctl daemon-reload
sudo systemctl restart penumbra tendermint

# view logs to monitor for errors
sudo journalctl -af -u penumbra -u tendermint
```

## Uninstalling
To remove the configs, run:

```bash
sudo systemctl disable --now penumbra tendermint
sudo rm /etc/systemd/system/{penumbra,tendermint}.service
sudo systemctl daemon-reload
```
