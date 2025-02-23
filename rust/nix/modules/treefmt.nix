{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
    inputs.flake-root.flakeModule
  ];
  perSystem =
    {
      self',
      inputs',
      pkgs,
      system,
      config,
      ...
    }:
    {
      treefmt = {
        inherit (config.flake-root) projectRootFile;
        programs.nixfmt.enable = true;
        programs.rustfmt.enable = true;
      };
    };
}
