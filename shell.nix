{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rust-analyzer
    pkgs.openssl
    pkgs.pkg-config
  ];

  shellHook = ''
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig"
  '';
}
