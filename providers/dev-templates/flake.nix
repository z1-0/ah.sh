{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f: nixpkgs.lib.genAttrs allSystems (system: f { pkgs = import nixpkgs { inherit system; }; });

      ahshLanguages = builtins.fromJSON (builtins.getEnv "AHSH_LANGUAGES");

      getTemplateShell =
        system: lang:
        (builtins.getFlake "github:the-nix-way/dev-templates?dir=${lang}").devShells.${system}.default;
    in
    {
      devShells = forAllSystems (
        { pkgs }:
        let
          system = pkgs.stdenv.hostPlatform.system;
          langShells = map (getTemplateShell system) ahshLanguages;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = langShells;
          };
        }
      );
    };
}
