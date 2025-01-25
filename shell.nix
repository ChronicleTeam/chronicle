{
  pkgs ? import <nixpkgs> { },
}:
with pkgs;
let
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
  ];

  RUSTUP_TOOLCHAIN = rustToolchain;
  RUSTUP_HOME = "${cache}/.rustup";
  CARGO_HOME = "${cache}/.cargo";

  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath [ stdenv.cc.cc ]}
  '';
}
