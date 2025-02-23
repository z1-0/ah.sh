{ inputs, ... }:
{
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
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        config.allowUnfree = true;
        config.allowUnfreePredicate = _: true;
      };
    };
}
