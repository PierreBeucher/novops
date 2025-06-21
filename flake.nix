{
  description = "Novops, the cross-platform secret manager for development and CI environments";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    
    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = (crane.mkLib nixpkgs.legacyPackages.${system});

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

        # awscli fails because of Python issue
        # See https://github.com/NixOS/nixpkgs/issues/267864#issuecomment-1865001204
        awscli2-patched = pkgs.awscli2.overrideAttrs (oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ pkgs.makeWrapper ];

          doCheck = false;

          # Run any postInstall steps from the original definition, and then wrap the
          # aws program with a wrapper that sets the PYTHONPATH env var to the empty
          # string
          postInstall = ''
            ${oldAttrs.postInstall}
            wrapProgram $out/bin/aws --set PYTHONPATH=
          '';
        });

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

          # Until https://github.com/NixOS/nixpkgs/issues/359286 fixed and https://github.com/NixOS/nixpkgs/pull/362898 merged
          # json-schema-for-humans
          (json-schema-for-humans.overrideAttrs (oldAttrs: {
            version = "1.3.0";
            src = fetchFromGitHub {
              owner = "coveooss";
              repo = "json-schema-for-humans";
              rev = "refs/tags/v1.3.0";
              hash = "sha256-0nen6oJOWdihm/EWKSGQLlD70pRxezhCHykBJxlSFHo=";
            };
            postPatch = ''
              substituteInPlace pyproject.toml \
                --replace-fail 'markdown2 = "^2.5.0"' 'markdown2 = "^2.4.1"'
            '';
          }))

          gnumake
          zip
          gh
          nodejs_24 # for test and release jobs
          nodePackages.pnpm
          cachix
          python311
          python311Packages.pip
          shellcheck
          yq
          azure-cli
          google-cloud-sdk
          go-task

          # Module testing
          podman
          podman-compose
          kind
          google-cloud-sdk
          bitwarden-cli
          sops
          age
          awscli2-patched
          aws-vault

          pulumi-bin
          pulumiPackages.pulumi-language-nodejs
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

            # Force use of localstack to avoid side effect during tests
            AWS_CONFIG_FILE = "${./tests/setup/aws/config}";
            AWS_SHARED_CREDENTIALS_FILE = "${./tests/setup/aws/credentials}";
            AWS_ENDPOINT_URL = "http://localhost:4566";
          };

          # Dev shell for cross-compilation with cross
          # Can't use directly in default shell: cross somehow conflicts with Crane
          cross = pkgs.mkShell {
            
            packages = devShellPackages ++ [
              pkgs.cargo-cross
              pkgs.rustup
              pkgs.nodejs_24 # for npx release-please
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