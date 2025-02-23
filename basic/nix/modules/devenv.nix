{ inputs, ... }:
{
  imports = [
    inputs.devenv.flakeModule
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
      devenv.shells.default = {
        packages = [
          pkgs.just
          config.treefmt.build.wrapper
        ]
        ++ builtins.attrValues config.treefmt.build.programs;

        git-hooks.hooks.treefmt = {
          enable = true;
          package = config.treefmt.build.wrapper;
        };

        languages.nix.enable = true;
      };
    };
}
