{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo-binstall
    pkgs.diesel-cli
    pkgs.gtk3
    pkgs.libsoup
    pkgs.openssl
    pkgs.pkg-config
    pkgs.rust-analyzer
    pkgs.sqlite
    pkgs.webkitgtk_4_1
    pkgs.xdotool
  ];

  shellHook = ''
    rustup target list | grep 'wasm32-unknown-unknown (installed)' || rustup target add wasm32-unknown-unknown
  '';
}
