# default.nix
{ pkgs ? import <nixpkgs> {} }:

let
  # Extract version from Cargo.toml
  version = builtins.readFile (pkgs.runCommand "version" {}
    ''
      ${pkgs.rust}/bin/cargo metadata --manifest-path=${toString ./Cargo.toml} \
        --format-version=1 \
        --no-deps \
        | ${pkgs.jq}/bin/jq -r '.packages[0].version' \
        > $out
    '');

in pkgs.rustPlatform.buildRustPackage {
  pname = "azadi-noweb";
  inherit version;

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with pkgs.lib; {
    description = "A Rust implementation of noweb-style literate programming tool";
    homepage = "https://github.com/giannifer7/azadi-noweb";
    license = licenses.mit;
    maintainers = with maintainers; [ giannifer7 ];
    mainProgram = "azadi-noweb";
  };
}
