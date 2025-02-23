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
      devenv.shells = {
        default = {
          languages.go.enable = true;
          languages.javascript.enable = true;
          languages.lua.enable = true;
          languages.nix.enable = true;
          languages.python.enable = true;
          languages.rust.enable = true;
          languages.shell.enable = true;
          packages = with pkgs; [
            # prettier
            nodePackages_latest.prettier
            stylua
            nixfmt
            black
          ];
        };
        c = {
          languages.c.enable = true;
        };
        go = {
          languages.go.enable = true;
        };
        haskell = {
          languages.haskell.enable = true;
        };
        java = {
          languages.java.enable = true;
        };
        js = {
          languages.javascript.enable = true;
          packages = [
            # prettier
            pkgs.nodePackages_latest.prettier
          ];
        };
        kotlin = {
          languages.kotlin.enable = true;
          packages = [ pkgs.ktlint ];
        };
        lua = {
          languages.lua.enable = true;
          packages = [ pkgs.stylua ];
        };
        nix = {
          languages.nix.enable = true;
          packages = [ pkgs.nixfmt ];
        };
        python = {
          languages.python.enable = true;
          packages = [ pkgs.black ];
        };
        rust = {
          languages.rust.enable = true;
        };
        scala = {
          languages.scala.enable = true;
        };
        ts = {
          languages.typescript.enable = true;
          packages = [
            # prettier
            pkgs.nodePackages_latest.prettier
          ];
        };
      };
    };
}
