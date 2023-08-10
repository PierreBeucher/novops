# Installation

## Linux

Novops is distributed as a standalone static binary. To install, run:

```sh
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

Install a specific version:

```sh
export NOVOPS_VERSION=0.6.0
curl -L "https://github.com/PierreBeucher/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

Novops is currently only available for x86-64 systems. More will come soon!

## Nix

```
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
              # Run novops on shel startup
              novops load -s .envrc && source .envrc
            '';
          };
        };
      }
    );    
}
```

## From source

Requirements:

- Make
- Docker 

Clone [Novops repository](https://github.com/PierreBeucher/novops) and run:

```
make build
```

Binary is built under `build/novops`

```
build/novops --version
```