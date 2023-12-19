{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.nci.url = "github:yusdacra/nix-cargo-integration";
  inputs.nci.inputs.nixpkgs.follows = "nixpkgs";
  inputs.parts.url = "github:hercules-ci/flake-parts";
  inputs.parts.inputs.nixpkgs-lib.follows = "nixpkgs";

  outputs =
    inputs @ { parts
    , nci
    , ...
    }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      imports = [ nci.flakeModule ];
      perSystem =
        { config
        , pkgs
        , ...
        }:
        let
          # shorthand for accessing outputs
          # you can access crate outputs under `config.nci.outputs.<crate name>` (see documentation)
          outputs = config.nci.outputs;
        in
        {
          # declare projects
          # TODO: change this to your workspace's path
          nci.projects."anyrun-plugins" = {
            path = ./.;
            # export all crates (packages and devshell) in flake outputs
            # alternatively you can access the outputs and export them yourself
            export = true;
          };
          # configure crates
          nci.crates = {
            "cliphist" = {
              # look at documentation for more options
            };
          };
          # export the project devshell as the default devshell
          devShells.default = outputs."anyrun-plugins".devShell;
          # export the release package of the crate as default package
          packages.default = outputs."cliphist".packages.release;
        };
    };
}
