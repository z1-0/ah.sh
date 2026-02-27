{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv.url = "github:cachix/devenv";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw= cachix.cachix.org-1:eWNHQldwUO7G2VkjpnjDbWwy4KQ/HNxht7H4SSoMckM=";
    extra-substituters = "https://devenv.cachix.org https://cachix.cachix.org";
  };

  outputs =
    { nixpkgs, devenv, ... }@inputs:
    let
      inherit (nixpkgs) lib;

      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f: lib.genAttrs allSystems (system: f { pkgs = import nixpkgs { inherit system; }; });
    in
    {
      devShells = forAllSystems (
        { pkgs }:
        let
          ahshLanguages = builtins.fromJSON (builtins.getEnv "AHSH_LANGUAGES");
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              (lib.optionalAttrs (ahshLanguages != [ ]) {
                languages = lib.genAttrs ahshLanguages (_: {
                  enable = true;
                });
              })
            ];
          };
        }
      );
    };
}
