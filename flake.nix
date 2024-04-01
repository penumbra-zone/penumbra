{
  description = "a nix development shell for penumbra";

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
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    cargo2nix {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs = {
          nixpkgs.follows = "nixpkgs";
          flake-utils.follows = "flake-utils";
        };
      }
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          buildInputs = with pkgs; [
            clang
            openssl
            rustToolchain
          ];
          code = pkgs.callPackage ./. { inherit pkgs crane rustToolchain buildInputs; };
        in
        with pkgs;
        rec {
          devShells.default = mkShell rec {
            inherit buildInputs nativeBuildInputs;
            shellHook = ''
              export CC="${pkgs.clang}/bin/clang"
              export CXX="${pkgs.clang}/bin/clang++"
              export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
            '';
          };

          packages = {
            pd = code.pd;
            pcli = code.pcli;
            pclientd = code.pclientd;
            all = pkgs.symlinkJoin {
              name = "all";
              paths = with code; [ pd pcli pclientd ];
            };
          };

          defaultPackage = self.packages.${system}.all;
        }
      );
}
