let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in nixpkgs.mkShell {
  buildInputs = with nixpkgs; [
    hidapi
    libusb1
    pkgconf
    latest.rustChannels.stable.rust
  ];
}
