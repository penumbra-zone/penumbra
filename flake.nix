{
  description = "A nix development shell and build environment for penumbra";

  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          # Define versions of Penumbra and CometBFT
          penumbraRelease = null; # Use the local working copy
          # To update the cometbft hash values, run:
          # nix-prefetch-git --url https://github.com/cometbft/cometbft --rev <tag>
          # and review the output.
          cometBftRelease = {
            version = "0.37.18";
            # Set `sha256` to the value `hash` in the nix-prefetch-git output.
            sha256 = "sha256-Gh5iGBlB5tFn1KCuF0nthb0me9Ts63quHKOYdNwmhIs=";
            # Set `vendorHash` to "", run `nix build`, and review the hash.
            vendorHash = "sha256-F6km3YpvfdpPeIJB1FwA5lQvPda11odny0EHPD8B6kw=";
          };

          # Build grpcui from source, for Reflection v1 support.
          # https://github.com/fullstorydev/grpcui/issues/322
          # To update the grpcui hash values, run:
          # nix-prefetch-git --url https://github.com/fullstorydev/grpcui --rev 483f037ec98b89200353c696d990324318f8df98
          grpcUiRelease = {
            version = "1.4.2-pre.1";
            sha256 = "sha256-3vjJNa1bXoMGZXPRyVqhxYZPX5FDp8Efy+w6gdx0pXE=";
            vendorHash = "sha256-j7ZJeO9vhjOoR8aOOJymDM6D7mPAJQoD4O6AyAsErRY=";
            rev = "483f037ec98b89200353c696d990324318f8df98";
          };

          # Set up for Rust builds, pinned to the Rust toolchain version in the Penumbra repository
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Add nightly Rust toolchain, required for building the rustdocs with combined index landing page.
          # https://github.com/oxalica/rust-overlay/blob/master/README.md#cheat-sheet-common-usage-of-rust-bin
          nightlyRustToolchain = pkgs.rust-bin.selectLatestNightlyWith(toolchain: toolchain.default);
          nightlyCraneLib = (crane.mkLib pkgs).overrideToolchain nightlyRustToolchain;

          # Important environment variables so that the build can find the necessary libraries
          PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
          ROCKSDB_LIB_DIR="${pkgs.rocksdb.out}/lib";
        in with pkgs; with pkgs.lib; let
          # Common development packages for all shells
          commonDevPackages = [
            buf
            cargo-hack
            cargo-nextest
            cargo-release
            cargo-watch
            cometbft
            dbus
            glibcLocales # for postgres initdb locale support
            grafana
            grpcui
            grpcurl
            just
            libusb1
            mdbook
            mdbook-katex
            mdbook-linkcheck
            mdbook-mermaid
            nix-prefetch-scripts
            postgresql
            process-compose
            prometheus
            protobuf
            rocksdb
            rsync
            sqlfluff
            toml-cli
          ];

          # Common shell hook content
          commonShellHook = ''
            export LIBCLANG_PATH=${LIBCLANG_PATH}
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc} # Required for rust-analyzer
            export ROCKSDB_LIB_DIR=${ROCKSDB_LIB_DIR}
            export RUST_LOG="info,network_integration=debug,pclientd=debug,pcli=info,pd=info,penumbra=info"
          '';

          # All the Penumbra binaries
          penumbra = (craneLib.buildPackage {
            pname = "penumbra";
            src = cleanSourceWith {
              src = if penumbraRelease == null then craneLib.path ./. else fetchFromGitHub {
                owner = "penumbra-zone";
                repo = "penumbra";
                rev = "v${penumbraRelease.version}";
                sha256 = "${penumbraRelease.sha256}";
              };
              filter = path: type:
                # Retain non-rust asset files as build inputs:
                # * no_lfs, param, bin: proving and verification parameters
                # * zip: frontend bundled assets
                # * sql: database schema files for indexing
                # * csv: default genesis allocations for testnet generation
                # * json: default validator info for testnet generation
                (builtins.match ".*\.(no_lfs|param|bin|zip|sql|csv|json)$" path != null) ||
                # ... as well as all the normal cargo source files:
                (craneLib.filterCargoSources path type);
            };
            nativeBuildInputs = [ pkg-config ];
            buildInputs = if stdenv.hostPlatform.isDarwin then 
              with pkgs.darwin.apple_sdk.frameworks; [clang openssl rocksdb SystemConfiguration CoreServices]
            else
              [clang openssl rocksdb];

            inherit system PKG_CONFIG_PATH LIBCLANG_PATH ROCKSDB_LIB_DIR;
            cargoExtraArgs = "-p pd -p pcli -p pclientd -p pindexer -p pmonitor";
            meta = {
              description = "A fully private proof-of-stake network and decentralized exchange for the Cosmos ecosystem";
              homepage = "https://penumbra.zone";
              license = [ licenses.mit licenses.asl20 ];
            };
          }).overrideAttrs (_: { doCheck = false; }); # Disable tests to improve build times

          # CometBFT
          cometbft = (buildGoModule rec {
            pname = "cometbft";
            version = cometBftRelease.version;
            subPackages = [ "cmd/cometbft" ];
            src = fetchFromGitHub {
              owner = "cometbft";
              repo = "cometbft";
              rev = "v${cometBftRelease.version}";
              hash = cometBftRelease.sha256;
            };
            vendorHash = cometBftRelease.vendorHash;
            meta = {
              description = "CometBFT (fork of Tendermint Core): A distributed, Byzantine fault-tolerant, deterministic state machine replication engine";
              homepage = "https://github.com/cometbft/cometbft";
              license = licenses.asl20;
            };
          }).overrideAttrs (_: { doCheck = false; }); # Disable tests to improve build times

          # grpcui
          grpcui = (buildGoModule rec {
            pname = "grpcui";
            version = grpcUiRelease.version;
            subPackages = [ "cmd/grpcui" ];
            src = fetchFromGitHub {
              owner = "fullstorydev";
              repo = "grpcui";
              rev = "${grpcUiRelease.rev}";
              hash = grpcUiRelease.sha256;
            };
            vendorHash = grpcUiRelease.vendorHash;
            meta = {
              description = "An interactive web UI for gRPC, along the lines of postman";
              homepage = "https://github.com/fullstorydev/grpcui";
              license = licenses.mit;
            };
          }).overrideAttrs (_: { doCheck = false; }); # Disable tests to improve build times
        in rec {
          packages = { inherit penumbra cometbft; };
          apps = {
            pd.type = "app";
            pd.program = "${penumbra}/bin/pd";
            pcli.type = "app";
            pcli.program = "${penumbra}/bin/pcli";
            pclientd.type = "app";
            pclientd.program = "${penumbra}/bin/pclientd";
            pindexer.type = "app";
            pindexer.program = "${penumbra}/bin/pindexer";
            pmonitor.type = "app";
            pmonitor.program = "${penumbra}/bin/pmonitor";
            cometbft.type = "app";
            cometbft.program = "${cometbft}/bin/cometbft";
          };
          defaultPackage = symlinkJoin {
            name = "penumbra-and-cometbft";
            paths = [ penumbra cometbft ];
          };
          devShells = {
            default = craneLib.devShell {
              inherit LIBCLANG_PATH ROCKSDB_LIB_DIR;
              inputsFrom = [ penumbra ];
              packages = commonDevPackages;
              shellHook = ''
                ${commonShellHook}
                echo "Using stable Rust from rust-toolchain.toml"
              '';
            };

            nightly = nightlyCraneLib.devShell {
              inherit LIBCLANG_PATH ROCKSDB_LIB_DIR;
              inputsFrom = [ penumbra ];
              packages = commonDevPackages;
              shellHook = ''
                ${commonShellHook}
                echo "Using nightly Rust toolchain"
              '';
            };
          };
        }
      );
}
