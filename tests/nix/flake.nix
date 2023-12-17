{
    description = "Minimal example Flake using Novops";

    nixConfig = {
        extra-substituters = [
            "https://novops.cachix.org"
        ];
        extra-trusted-public-keys = [
            "novops.cachix.org-1:xm1fF2MoVYRmg89wqgQlM15u+2bk0LBfVktN9EgDaHY="
        ];
    };
    
    inputs = {
        novops.url = "github:PierreBeucher/novops/nix-cargo-crane"; 
        flake-utils.url = "github:numtide/flake-utils";
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    };

    outputs = { self, nixpkgs, novops, flake-utils }:
        flake-utils.lib.eachDefaultSystem (system: 
        let
            novopsPackage = novops.packages.${system}.novops; # Novops package must exists
            novopsDefaultPackage = novops.packages.${system}.default; # default Novops package must exists
            pkgs = nixpkgs.legacyPackages.${system};      
        in {
            packages = {
                default = novopsDefaultPackage;
                novops = novopsPackage;
            };

            devShells = {
                default = nixpkgs.legacyPackages.${system}.mkShell {
                    packages = [ 
                        novopsPackage
                        novopsDefaultPackage
                    ];
                    shellHook = ''
                        # Run novops on shell startup
                        source <(novops load)
                    '';
                };
            };
        });
}