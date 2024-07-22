# Developer environment

To get started working on Penumbra, you'll need a few dependencies on your workstation.
Running tests and local services is more involved. The project uses [Nix] to automate
the creation of developer environments with suitable tooling. If you'd prefer not to use
Nix, and instead configure your environment manually, see the docs on
[compiling from source](./build.md).

## Installation OS-level packages
You'll need `git` and `git-lfs` to clone the Penumbra protocol repository.
Install these via your package manager of choice:

```bash
# for macos
brew install git-lfs
# for linux debian/ubuntu
sudo apt install -y git-lfs
# for linux fedora
sudo dnf install -y git-lfs
```

Then, for all platforms, make sure to run `git lfs install`. Now you're ready to clone the
Penumbra protocol repo:

```
git clone https://github.com/penumbra-zone/penumbra
```

## Using `nix develop` for project dependencies

<!--
The Informal Systems team has a Nix setup at https://github.com/informalsystems/cosmos.nix,
which recommends using the NixOS installer, which is why we recommend using that installer, too.
-->

Install [Nix]. After restarting your shell, create a config file to enable
Nix flakes:

```
mkdir -p ~/.config/nix
echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
```

Now hop into the Penumbra directory and activate the env:

```
cd penumbra
nix develop
```

You'll have to wait a bit for packages to be built and installed. Once it finishes,
your active shell will have access to Penumbra project dependencies, like a compatible
version of `cometbft`, and other dev tooling, like `grpcurl` and `mdbook`.
You can run `exit` to return to your normal shell, without those tools, which have been installed to `/nix/store/`.

## Using `direnv`

If you use [direnv], you can copy the example `.envrc` file to automatically activate
the Penumbra nix environment when you cd to the repository:

```
cp .envrc.example .envrc
direnv allow
```

The `.envrc` path is intentionally git-ignored, so you can customize it as you see fit.
If you don't use `direnv`, you'll need to run `nix develop` in any terminal window
where you want access to the Penumbra dev env. Using `direnv` will make shell startup
a bit slower, so choose what's best for you.

[Nix]: https://nixos.org/
[direnv]: https://direnv.net/
