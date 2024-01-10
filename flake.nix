{
	description = "A lightweight streamdeck controller with plugin support";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
		rust-overlay.url = "github:oxalica/rust-overlay";
	};

	outputs = inputs: let
		system = "x86_64-linux";
		pkgs = import inputs.nixpkgs {
			overlays = [inputs.rust-overlay.overlays.default];
			inherit system;
		};
		
	in {
		packages.${system} = {
			default = pkgs.callPackage pkgs.rustPlatform.buildRustPackage rec {
				pname = "Rstreamdeck";
				version = "0.0.1-alpha";
				
				src = ./.;
				cargoLock.lockFile = ./Cargo.lock;
				buildAndTestSubdir = "streamdeck-control";


				nativeBuildInputs = with pkgs; [
					hidapi
					pkgconf
					libusb1
					dbus
				];
			};
		};

		devShells.${system} = {
			default = pkgs.mkShell {
				packages = with pkgs; [
					hidapi
					pkgconf
					libusb1
					rust-bin.stable.latest.default
					rust-analyzer
					lldb
					dbus
				];
			};
		};
	};
}
