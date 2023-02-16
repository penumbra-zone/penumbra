# systemd unit files for Penumbra

Here are example unit files for running Penumbra (and the required Tendermint instance)
via systemd. You'll need to customize the `User` declaration, and possibly the path
to the home directory, as well, depending on your system. The paths to the binaries,
in the `ExecStart` lines, assume that a symlink exists to the locally compiled versions,
as described in the [install guide](https://guide.penumbra.zone/main/pd/build.html).

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
