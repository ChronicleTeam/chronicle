{
  pkgs ? import <nixpkgs> { },
}:
with pkgs;
let
  unstable = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") { };
  cache = toString ./.nix-files;
  rustToolchain = "stable";
  nodejs = nodejs_22;
in
mkShell rec {

  buildInputs = [
    openssl
    pkg-config
    rustup
    nodejs
    nodePackages.npm
    nodePackages.vercel
    unstable.cargo-shuttle
    postgresql_17
  ];

  RUSTUP_TOOLCHAIN = rustToolchain;
  RUSTUP_HOME = "${cache}/.rustup";
  CARGO_HOME = "${cache}/.cargo";
  
  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath [ stdenv.cc.cc ]}
  '';
}
