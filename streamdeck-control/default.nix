{rustPlatform
, hidapi    
, libusb1
, pkgconf
, lib
}: {
	Rstreamdeck = rustPlatform.buildRustPackage rec {
			pname = "Rstreamdeck";
			version = "0.1.0";
	
			src = "./";

			cargoHash = "";

				buildInputs = [
					hidapi
					libusb1
					pkgconf
				];

			meta = with lib; {
				description = "";
				homepage = "";
				license = licenses.mit;
			};
		};
}
