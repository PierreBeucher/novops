{
  description = "Novops, the cross-platform secret manager for development and CI environments";

  # Flake config inspired from https://github.com/srid/rust-nix-template/blob/master/flake.nix
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          rust-toolchain = pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = [ pkgs.rustc pkgs.cargo pkgs.cargo-watch pkgs.rust-analyzer pkgs.rustPlatform.rustcSrc ];
          };

        in {
          
          # Rust package
          packages.novops = pkgs.rustPlatform.buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "vaultrs-0.7.0" = "sha256-aRbduZEQQ+4Rmk/g757yiZ8IkWemoALcPjHh9Q5tLTU=";
              };
            };

            doCheck = false;

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl.dev
            ];
          };

          packages.default = packages.novops;

          # Rust dev environment
          devShells.default = pkgs.mkShell {
            shellHook = ''
              # For rust-analyzer 'hover' tooltips to work.
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            '';
            
            buildInputs = with pkgs; [
              pkg-config
              openssl.dev
              mdbook
              mdbook-linkcheck
              google-cloud-sdk
              bitwarden-cli
              json-schema-for-humans
              podman
              podman-compose
              gnumake
              zip
              gh
              nodejs-slim # for npx release-please
            ];
            nativeBuildInputs = with pkgs; [
              rust-toolchain
            ];
          };


        };
    };
}