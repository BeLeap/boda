{
  description = "opinionated alternative watch";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = rec {
          boda = pkgs.rustPlatform.buildRustPackage {
            pname = "boda";
            version = "0.2526.0";
            src = ./.;

            cargoBuildFlags = "-p boda";

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
          default = boda;
        };
        devShells = import ./shell.nix { inherit pkgs; };
      }
    );
}
