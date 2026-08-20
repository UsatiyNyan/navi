{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    layer = {
      url = "git+https://codeberg.org/us4tiyny4n/14y3r.git";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    layer,
    ...
  }: {
    devShells = layer.lib.forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = layer.lib.mkComposedShell {
        inherit system;
        alias = "layer";
        bases = ["nix" "rust" "js"];
        extraBuildInputs = with pkgs; [
          just
        ];
        extraShellHook = ''
        '';
        extraNixvimComponents = [
          ({...}: {
          })
        ];
      };
    });
  };
}
