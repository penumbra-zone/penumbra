{
  description = "A nix development shell and build environment for penumbra";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
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
              glibcLocales # for postgres initdb locale support
              cometbft
              grafana
              grpcurl
              grpcui
              just
              mdbook
              mdbook-katex
              mdbook-mermaid
              mdbook-linkcheck
              nix-prefetch-scripts
              postgresql
              process-compose
              prometheus
              protobuf
              rocksdb
              rsync
              sqlfluff
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
