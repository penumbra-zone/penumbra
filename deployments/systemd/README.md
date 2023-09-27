# systemd unit files for Penumbra

Here are example unit files for running Penumbra (and the required CometBFT instance)
via systemd. *You'll need to customize the the service files*, particularly the `User` declaration
and binary fullpath in `ExecStart`, and possibly the path to the home directory, depending on your system.

## Installing
Copy the service files to a system-wide location:

```bash
# use 'envsubst' to replace `$USER` with your local username
envsubst < penumbra.service | sudo tee /etc/systemd/system/penumbra.service
envsubst < cometbft.service | sudo tee /etc/systemd/system/cometbft.service
sudo systemctl daemon-reload
sudo systemctl restart penumbra cometbft

# view logs to monitor for errors
sudo journalctl -af -u penumbra -u cometbft
```

## Uninstalling
To remove the configs, run:

```bash
sudo systemctl disable --now penumbra cometbft
sudo rm /etc/systemd/system/{penumbra,cometbft}.service
sudo systemctl daemon-reload
```
