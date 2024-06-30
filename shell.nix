{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
      gcc
      pkg-config
      cairo
      graphene
      glib
      gtk4
      gtk4-layer-shell
    ];
}
