# Nix

Setup a development shell with [Nix Flakes](https://nixos.wiki/wiki/Flakes).

Add Novops as input:

```nix
  inputs = {
    # ...
    novops.url = "github:novadiscovery/novops";
    # ...
  };
```

And then include Novops package wherever needed.

Example minimal Flake:

```nix
{
    description = "Minimal example Flake using Novops";

    inputs = {
        novops.url = "github:novadiscovery/novops"; 
    };

    outputs = { self, nixpkgs, novops }: {
        devShells."x86_64-linux".default = nixpkgs.legacyPackages."x86_64-linux".mkShell {
            packages = [ 
                novops.packages."x86_64-linux".novops
            ];
            shellHook = ''
                # Run novops on shell startup
                novops load -s .envrc && source .envrc
            '';
        };
    };
}
```

A more complete Flake using [`flake-utils`](https://github.com/numtide/flake-utils):

```nix
{
  description = "Example Flake using Novops";

  inputs = {
    novops.url = "github:novadiscovery/novops"; # Add novops input
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, novops, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
        pkgs = nixpkgs.legacyPackages.${system};
        novopsPackage = novops.packages.${system}.novops; 
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = [ 
              novopsPackage # Include Novops package in your shell
            ];
            shellHook = ''
              # Run novops on shell startup
              novops load -s .envrc && source .envrc
            '';
          };
        };
      }
    );    
}
```