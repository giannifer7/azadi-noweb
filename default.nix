# default.nix
{ pkgs ? import <nixpkgs> {
    config = {
      allowUnfree = true;
      permittedInsecurePackages = [];
    };
  }
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "azadi-noweb";
  version = "0.1.2";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  # Skip tests during the build
  doCheck = false;

  meta = with pkgs.lib; {
    description = "A Rust implementation of noweb-style literate programming tool";
    homepage = "https://github.com/giannifer7/azadi-noweb";
    license = licenses.mit;
    maintainers = with maintainers; [ giannifer7 ];
    mainProgram = "azadi-noweb";
  };
}
