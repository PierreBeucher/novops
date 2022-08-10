{
  description = "Novops flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (system: let

      pkgs = nixpkgs.legacyPackages.${system}.pkgs;

    in rec {

      packages = rec {
        default = novops;
        novops = pkgs.rustPlatform.buildRustPackage {
          pname = "novops";
          version = "0.1.2";

          # this copies the whole folder, there is probably a better solution
          src = ./.;

          cargoSha256 = "sha256-zpbvBc9lKuGODyTIVGWMan1/B9B2V7Vi/0QyhrznfGM=";
        };

      };

      # deprecated in recent nix ~ > 2.8
      defaultPackage = packages.novops;
  });
}
