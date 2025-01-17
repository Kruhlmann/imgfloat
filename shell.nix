{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.diesel-cli
    pkgs.libpqxx
    pkgs.sqlite
    pkgs.openssl
    pkgs.pkg-config
    pkgs.rust-analyzer
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
  '';
}
