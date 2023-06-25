{
  description = "Novops flake";

  inputs = {
    dream2nix.url = "github:nix-community/dream2nix";
    nixpkgs.follows = "dream2nix/nixpkgs";
  };

  outputs = { self, nixpkgs, dream2nix }:
    let 
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in 
      dream2nix.lib.makeFlakeOutputs {
        systems = [ system ];
        
        config.projectRoot = ./.;
        source = ./.;
        projects = ./projects.toml;

        packageOverrides = {

          # Add build dependencies
          # dream2nix uses crane and generate 2 derivations novops and novops-deps
          # Each needs additional build inputs
          # See https://nix-community.github.io/dream2nix/subsystems/rust.html#override-gotchas
          novops-deps = {
            add-deps = {
              nativeBuildInputs = old: old ++ [
                pkgs.pkg-config
                pkgs.openssl.dev
              ];
            };
          };

          novops = {
            add-deps = {
              buildInputs = [
                pkgs.openssl.dev
                pkgs.pkg-config
                pkgs.mdbook
                pkgs.google-cloud-sdk
              ];

              # Skip tests as most are integration tests requiring setup
              # Tests are run outside of Nix context for now 
              doCheck = false;
            };
          };
        };
      };
}