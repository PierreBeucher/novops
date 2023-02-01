{
  description = "Novops flake";

  inputs = {
    # cargo2nix is a more granular version of nixpkgs' buildRustPackage
    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = { self, cargo2nix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.66.1";
          packageFun = import ./Cargo.nix;
        };

      in rec {
        devShells = {
          default = self.packages.${system}.novops.overrideAttrs(oa: {
            buildInputs = oa.buildInputs ++ [ 
              cargo2nix.packages.${system}.cargo2nix
              pkgs.pkg-config
              pkgs.openssl.dev # required by openssl crate (transitive dep) for cargo build
            ];
          });
        };

        packages = {
          novops = (rustPkgs.workspace.novops {}).bin;
          default = packages.novops;
        };
      }
    );
}
