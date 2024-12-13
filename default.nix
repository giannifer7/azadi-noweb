# default.nix
{ pkgs ? import <nixpkgs> {
    config = {
      allowUnfree = true;
      permittedInsecurePackages = [];
    };
  }
}:

let
  naersk = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "naersk";
    rev = "aeb58d5e8faead8980a807c840232697982d47b9";
    sha256 = "sha256:0l2me2j8i4zvm3hg5fd8j27qg4bjgvj3zm9nvxqz2q1qg2r8584g";
  }) {};
in

naersk.buildPackage {
  pname = "azadi-noweb";
  version = "0.1.2";

  src = ./.;

  buildInputs = with pkgs; [
    pkg-config
  ];

  overrideMain = attrs: {
    # Skip tests during the build
    doCheck = false;

    # Add any necessary build inputs
    nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ (with pkgs; [
      rustc
      cargo
    ]);
  };

  meta = with pkgs.lib; {
    description = "A Rust implementation of noweb-style literate programming tool";
    homepage = "https://github.com/giannifer7/azadi-noweb";
    license = licenses.mit;
    maintainers = with maintainers; [ giannifer7 ];
    mainProgram = "azadi-noweb";
  };
}
