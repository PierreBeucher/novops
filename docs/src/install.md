# Installation

- [Linux](#linux)
- [MacOS (Darwin)](#macos-darwin)
- [Windows](#windows)
- [Arch Linux](#arch-linux)
- [Nix](#nix)
- [Direct binary download](#direct-binary-download)
- [Build from source](#build-from-source)
  - [Updating](#updating)

Novops is distributed as a standalone static binary. No dependencies are required.

## Linux

Download latest Novops binary latest version:

```sh
# x86-64
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops_linux_x86_64.zip" -o novops.zip

# arm64
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops_linux_aarch64.zip" -o novops.zip
```

Or specific version:

```sh
NOVOPS_VERSION=v0.12.0

# x86-64
curl -L "https://github.com/PierreBeucher/novops/releases/download/${NOVOPS_VERSION}/novops_linux_x86_64.zip" -o novops.zip

# arm64
curl -L "https://github.com/PierreBeucher/novops/releases/download/${NOVOPS_VERSION}/novops_linux_aarch64.zip" -o novops.zip
```

Install it:

```sh
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

Check it works:

```sh
novops --version
```

## MacOS (Darwin)

Download latest Novops binary latest version:

```sh
# x86-64
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops_macos_x86_64.zip" -o novops.zip

# arm64
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops_macos_aarch64.zip" -o novops.zip
```

Or specific version:

```sh
NOVOPS_VERSION=v0.12.0

# x86-64
curl -L "https://github.com/PierreBeucher/novops/releases/download/${NOVOPS_VERSION}/novops_macos_x86_64.zip" -o novops.zip

# arm64
curl -L "https://github.com/PierreBeucher/novops/releases/download/${NOVOPS_VERSION}/novops_macos_aarch64.zip" -o novops.zip
```

Install it:

```sh
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

Check it works:

```sh
novops --version
```

## Windows

Novops does not offer native Windows ([coming soon](https://github.com/PierreBeucher/novops/issues/90)). You can use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) in the meantime, following Linux installation.

## Arch Linux

Available in the AUR (Arch User Repository)

```sh
yay -S novops-git
```

## Nix

Use a `flake.nix` such as:

```nix
{
  description = "Example Flake using Novops";

  # Optional: use Cachix cache to avoid re-building Novops
  nixConfig = {
    extra-substituters = [
      "https://novops.cachix.org"
    ];
    extra-trusted-public-keys = [
      "novops.cachix.org-1:xm1fF2MoVYRmg89wqgQlM15u+2bk0LBfVktN9EgDaHY="
    ];
  };
    
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
              # Run novops on shel startup
              novops load -s .envrc && source .envrc
            '';
          };
        };
      }
    );
}
```

## Direct binary download

See [GithHub releases](https://github.com/PierreBeucher/novops/releases) to download binaries directly.

## Build from source

See [Development and contribution guide](contributing/development.md) to build from source.

### Updating

To update Novops, replace binary with a new one following installation steps above.
