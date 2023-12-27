{
  description = "Novops, the cross-platform secret manager for development and CI environments";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};        
        craneLib = crane.lib.${system};

        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          
          strictDeps = true;
          doCheck = false;

          buildInputs = [
            pkgs.openssl
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };

        novopsPackage = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
        
      in {

        packages = {
          default = novopsPackage;
          novops = novopsPackage;
        };

        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            # Dev tools
            pkg-config
            openssl.dev
            mdbook
            mdbook-linkcheck
            json-schema-for-humans
            gnumake
            zip
            gh
            nodejs-slim # for npx release-please
            cachix

            # Module testing
            podman
            podman-compose
            google-cloud-sdk
            bitwarden-cli
            sops
            age 
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    );
}