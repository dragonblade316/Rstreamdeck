let
  rust_overlay = import (builtins.fetchTarball https://github.com/oxalica/rust-overlay/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
in nixpkgs.mkShell {
  buildInputs = with nixpkgs; [
		dbus
    hidapi
    libusb1
    pkgconf
    rust-bin.stable.latest.default
    rust-analyzer
    lldb
    protobuf
  ];
}
