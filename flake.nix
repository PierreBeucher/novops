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
          version = "0.1.0";

          # this copies the whole folder, there is probably a better solution
          src = ./.;

          cargoSha256 = "sha256-GptJwfHfQd7VReS0tXyS5f33BYeI1z5OGyPhWUKzm+A=";
        };

      };

      defaultPackage = packages.novops;

      defaultApp = flake-utils.lib.mkApp { name = "novops"; drv = packages.novops;};
  });
}
