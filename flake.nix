{
	description = "A lightweight streamdeck controller with plugin support";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";
		rust-overlay.url = "github:oxalica/rust-overlay";
	};

	outputs = inputs: let
		system = "x86_64-linux";
		pkgs = import inputs.nixpkgs {
			overlays = [inputs.rust-overlay.overlays.default];
		};
		
	in {
		packages.${system} = {
			default = pkgs.callPackage ./streamdeck-control {};
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
				];
			};
		};
	};
}
