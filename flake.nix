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
          # penumbraRelease = { # Use a specific release
          #   version = "0.71.0";
          #   sha256 = "sha256-2mpyBEt44UlXm6hahJG9sHGxj6nzh7z9lnj/vLtAAzs=";
          # };
          cometBftRelease = {
            version = "0.37.5";
            sha256 = "sha256-wNVHsifieAtZgedavCEJLgG0kRDqUhG4Lk5ciTPoNzI=";
            vendorHash = "sha256-JPEGMa0HDesEtKFvgLUP2UfTB0DlParepE2p+n06Igc=";
          };

          # Set up for Rust builds
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          craneLib = crane.mkLib pkgs;

          # Important environment variables so that the build can find the necessary libraries
          PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
        in with pkgs; with pkgs.lib; let
          # All the Penumbra binaries
          penumbra = (craneLib.buildPackage {
            pname = "penumbra";
            src = cleanSourceWith {
              src = if penumbraRelease == null then craneLib.src "./." else fetchFromGitHub {
                owner = "penumbra-zone";
                repo = "penumbra";
                rev = "v${penumbraRelease.version}";
                sha256 = "${penumbraRelease.sha256}";
              };
              filter = path: type:
                (builtins.match ".*\.(no_lfs|param||bin)$" path != null) ||
                (craneLib.filterCargoSources path type);
            };
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ clang openssl ];
            inherit system PKG_CONFIG_PATH LIBCLANG_PATH;
            cargoExtraArgs = "-p pd -p pcli -p pclientd";
            meta = {
              description = "A fully private proof-of-stake network and decentralized exchange for the Cosmos ecosystem";
              homepage = "https://penumbra.zone";
              license = [ licenses.mit licenses.asl20 ];
            };
          }).overrideAttrs (_: { doCheck = false; });

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
          }).overrideAttrs (_: { doCheck = false; });
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
            inherit LIBCLANG_PATH;
            inputsFrom = [ penumbra ];
            packages = [ cargo-watch ];
          };
        }
      );
}
