{
  description = "A nix development shell and build environment for penumbra";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = { nixpkgs.follows = "nixpkgs"; };
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
            version = "0.37.9";
            sha256 = "sha256-4LUdDlDog4kbiwyGo5fZEvtDXa6sIm+SKlSBWq1angc=";
            vendorHash = "sha256-0iqI/Z8rqDyQ7JqSrsqA9kADqF6qZy8NxTDNjAYYHts=";
          };

          # Set up for Rust builds, pinned to the Rust toolchain version in the Penumbra repository
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Important environment variables so that the build can find the necessary libraries
          PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
          ROCKSDB_LIB_DIR="${pkgs.rocksdb.out}/lib";
        in with pkgs; with pkgs.lib; let
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
                # Retain proving and verification parameters, and no-lfs marker file ...
                (builtins.match ".*\.(no_lfs|param||bin)$" path != null) ||
                # ... as well as all the normal cargo source files:
                (craneLib.filterCargoSources path type);
            };
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ clang openssl rocksdb ];
            inherit system PKG_CONFIG_PATH LIBCLANG_PATH ROCKSDB_LIB_DIR;
            cargoExtraArgs = "-p pd -p pcli -p pclientd";
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
        in rec {
          packages = { inherit penumbra cometbft; };
          apps = {
            pd.type = "app";
            pd.program = "${penumbra}/bin/pd";
            pcli.type = "app";
            pcli.program = "${penumbra}/bin/pcli";
            pclientd.type = "app";
            pclientd.program = "${penumbra}/bin/pclientd";
            cometbft.type = "app";
            cometbft.program = "${cometbft}/bin/cometbft";
          };
          defaultPackage = symlinkJoin {
            name = "penumbra-and-cometbft";
            paths = [ penumbra cometbft ];
          };
          devShells.default = craneLib.devShell {
            inherit LIBCLANG_PATH ROCKSDB_LIB_DIR;
            inputsFrom = [ penumbra ];
            packages = [
              cargo-nextest
              cargo-watch
              cometbft
              just
              nix-prefetch-scripts
              postgresql
              protobuf
              rocksdb
            ];
            shellHook = ''
              export LIBCLANG_PATH=${LIBCLANG_PATH}
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc} # Required for rust-analyzer
              export ROCKSDB_LIB_DIR=${ROCKSDB_LIB_DIR}
            '';
          };
        }
      );
}
