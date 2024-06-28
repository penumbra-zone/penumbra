# systemd unit files for Penumbra

Here are example unit files for running Penumbra (and the required CometBFT instance)
via systemd. *You'll need to customize the service files*, particularly the `User` declaration
and binary fullpath in `ExecStart`, and possibly the path to the home directory, depending on your system.

See the [guide] for details on how to use these configs.

[guide]: https://guide.penumbra.zone
