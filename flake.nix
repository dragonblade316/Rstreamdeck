{
	description = "A lightweight streamdeck controller with plugin support";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
		# rust-overlay.url = "github:oxalica/rust-overlay";
	};

	outputs = inputs: let
		system = "x86_64-linux";
		pkgs = import inputs.nixpkgs {
			# overlays = [inputs.rust-overlay.overlays.default];
			inherit system;
		};

		rstreamdeck = {
			default = pkgs.callPackage pkgs.rustPlatform.buildRustPackage rec {
				pname = "Rstreamdeck";
				version = "0.0.1-alpha";
				
				src = ./.;
				cargoLock.lockFile = ./Cargo.lock;
				buildAndTestSubdir = "streamdeck-control";


				buildInputs = with pkgs; [
					hidapi
					pkgconf
					libusb1
					dbus
				];

				nativeBuildInputs = with pkgs; [
					pkg-config
				];


			};
		};

		
	in {
		packages.${system} = rstreamdeck; 

		overlays = (prev: final: {
			rstreamdeck = final.callPackage rstreamdeck {}; 
		});


		devShells.${system} = {
			default = pkgs.mkShell {
				packages = with pkgs; [
					hidapi
					pkgconf
					libusb1
					cairo
					dbus
					cargo
					rustc
					rust-analyzer
					lldb
				];
			};
		};
	};
}
