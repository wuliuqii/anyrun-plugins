{
  description = "My custom anyrun plugins";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs @ { flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem =
        { config
        , pkgs
        , ...
        }:
        let
          lockFile = ./Cargo.lock;
        in
        {
          formatter = pkgs.nixpkgs-fmt;

          packages = {
            # expose each plugin as a package
            applications = pkgs.callPackage ./plugin.nix {
              inherit inputs lockFile;
              name = "applications";
            };

            cliphist = pkgs.callPackage ./plugin.nix {
              inherit inputs lockFile;
              name = "cliphist";
            };

            hyprwin = pkgs.callPackage ./plugin.nix {
              inherit inputs lockFile;
              name = "hyprwin";
            };

            symbols = pkgs.callPackage ./plugin.nix {
              inherit inputs lockFile;
              name = "symbols";
            };
          };
        };
    };
}
