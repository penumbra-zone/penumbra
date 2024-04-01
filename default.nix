{ pkgs ? import <nixpkgs> {}, crane, system, rustToolchain, buildInputs, ... }:

let
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "f4jumble-0.0.0" = "sha256-Zc07fVIPuGVq6gUW1OOehr7HkcCBNUPYNh7POmRECrE=";
    };
  };
  nativeBuildInputs = [ pkgs.pkg-config ];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  buildCrate = name:
    craneLib.buildPackage {
      pname = name;
      inherit src nativeBuildInputs buildInputs;
      preConfigurePhases = [ "addEnvVars" ];
      addEnvVars = ''
        export PKG_CONFIG_PATH="${PKG_CONFIG_PATH}"
        export LIBCLANG_PATH="${LIBCLANG_PATH}"
      '';
    };
in {
  pd = buildCrate "pd";
  pcli = buildCrate "pcli";
  pclientd = buildCrate "pclientd";
}