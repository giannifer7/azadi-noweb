# default.nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "azadi-noweb";
  version = "0.1.2";  # This should match your Cargo.toml

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  # Don't run tests in parallel
  cargoParallelTestThreads = 1;

  # Skip the cargo config to avoid metadata parsing issues
  CARGO_CONFIG = "";

  meta = with pkgs.lib; {
    description = "A Rust implementation of noweb-style literate programming tool";
    homepage = "https://github.com/giannifer7/azadi-noweb";
    license = licenses.mit;
    maintainers = with maintainers; [ giannifer7 ];
    mainProgram = "azadi-noweb";
  };
}
