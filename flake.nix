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
        novops = pkgs.rustPlatform.buildRustPackage rec {
          pname = "novops";
          version = (pkgs.lib.importTOML ./Cargo.toml).package.version;

          # by default the vendored archive is named after the version and since 
          # gitlab bumps the version number on every Merge Request it's best
          # not to depend on version for the hash
          # https://nixos.org/manual/nixpkgs/stable/#rust
          cargoDepsName = pname;

          # this copies the whole folder, there is probably a better solution
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          # cargoHash = "sha256-0/EypDUIhWZGZ99siF2QhY9KnZ4yfeljr+BIIKRjsg0=";
        };

      };

      # deprecated in recent nix ~ > 2.8
      defaultPackage = packages.novops;
  });
}
