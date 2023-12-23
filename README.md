# My own custom [anyrun](https://github.com/Kirottu/anyrun) plugins

Using helix's new fuzzy matcher [nucleo](https://github.com/helix-editor/nucleo).

## Installation

Add the flake:
```nix
# flake.nix
{
  inputs = {
    ...

    anyrun-plugins = {
      url = "github:wuliuqii/anyrun-plugins";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    ...
  };
}
```

The flake provides multiple packages:

- cliphist - the cliphist plugin
- applications - the applications plugin

Add to anyrun's home-manager module:
```nix
{
  programs.anyrun = {
    enable = true;
    config = {
      plugins = [
        ...
        "${inputs.anyrun-plugins.packages.${pkgs.system}.cliphist"
        ...
      ];
      ...
    };
  };
}
```

## Plugins

- [Cliphist](./cliphist/README.md)
  - Find clipboard history using the [cliphist](https://github.com/sentriz/cliphist).

- [Applications](./applications/README.md)
  - Launch applications, originally from [anyrun](https://github.com/Kirottu/anyrun/tree/master/plugins/applications), but with nucleo.