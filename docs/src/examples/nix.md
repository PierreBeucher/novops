# Nix

Setup a development shell with [Nix Flakes](https://nixos.wiki/wiki/Flakes).

Add Novops as input:

```nix
  inputs = {
    novops.url = "github:PierreBeucher/novops";
  };
```

And then include Novops package wherever needed.

Example `flake.nix`:

```nix
{
    inputs = {
        novops.url = "github:PierreBeucher/novops"; 
    };

    outputs = { self, nixpkgs, novops }: {
        devShells."x86_64-linux".default = nixpkgs.legacyPackages."x86_64-linux".mkShell {
            packages = [ 
                novops.packages."x86_64-linux".novops
            ];
            shellHook = ''
                # Run novops on shell startup
                source <(novops load)
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
    novops.url = "github:PierreBeucher/novops"; # Add novops input
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
              source <(novops load)
            '';
          };
        };
      }
    );    
}
```