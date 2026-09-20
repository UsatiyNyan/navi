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
          lld

          # graphics
          wayland
          wayland-scanner
          wayland-protocols
          libxkbcommon
          libffi
          libglvnd
          vulkan-loader
          vulkan-headers
        ];
        extraShellHook = ''
          export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.vulkan-loader}/lib:/run/opengl-driver/lib:$LD_LIBRARY_PATH"
          export XDG_DATA_DIRS="/run/opengl-driver/share:$XDG_DATA_DIRS"
        '';
        extraNixvimComponents = [
          ({...}: {
            plugins = {
              lsp.servers.glsl_analyzer.enable = true;
              lsp.servers.wgsl_analyzer.enable = true;

              treesitter.settings.ensure_installed = [
                "glsl"
                "wgsl"
                "fbs"
              ];

              # lsp.servers.rust_analyzer.settings.cargo.target = "wasm32-unknown-unknown";
            };
          })
        ];
      };
    });
  };
}
