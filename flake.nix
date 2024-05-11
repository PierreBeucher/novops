{
  description = "Novops, the cross-platform secret manager for development and CI environments";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay, ... }:
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

        # Check if current system is Darwin (Mac)
        # to add packages and buildInputs below
        isDarwin = system == "x86_64-darwin" || system == "aarch64-darwin";

        devShellPackages = with pkgs; [
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
          python311
          python311Packages.pip
          shellcheck

          # Module testing
          podman
          podman-compose
          google-cloud-sdk
          bitwarden-cli
          sops
          age
        ];

        devShellBuildInputs = with pkgs; [] ++ lib.optionals isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.libiconv
        ];
        
      in {

        packages = {
          default = novopsPackage;
          novops = novopsPackage;
        };

        devShells = {
          default = craneLib.devShell {
            packages = devShellPackages;
            buildInputs = devShellBuildInputs;

            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };

          # Dev shell for cross-compilation with cross
          # Can't use directly in default shell: cross somehow conflicts with Crane
          cross = pkgs.mkShell {
            
            packages = devShellPackages ++ [
              pkgs.cargo-cross
              pkgs.rustup
            ];

            buildInputs = devShellBuildInputs;

          };

          # Dev shell with Nightly Rust
          # Inspired from https://github.com/ipetkov/crane/blob/afdcd41180e3dfe4dac46b5ee396e3b12ccc967a/examples/build-std/flake.nix
          nightly = let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            };

            rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" ];
              targets = [ "x86_64-unknown-linux-gnu" ];
            });

            # NB: we don't need to overlay our custom toolchain for the *entire*
            # pkgs (which would require rebuidling anything else which uses rust).
            # Instead, we just want to update the scope that crane will use by appending
            # our specific toolchain there.
            craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          in craneLib.devShell {
            packages = devShellPackages;
            buildInputs = devShellBuildInputs;
            
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        };
      }
    );
}