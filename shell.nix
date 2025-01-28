{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.diesel-cli
    pkgs.libpqxx
    pkgs.sqlite
    pkgs.openssl
    pkgs.pkg-config
    pkgs.gcc
    pkgs.rust-analyzer
    pkgs.cargo-binstall
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    command -v dx || yes | cargo binstall dioxus-cli
  '';
}
