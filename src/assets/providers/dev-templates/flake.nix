{
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

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

      getTemplateShell =
        system: lang:
        (builtins.getFlake "github:the-nix-way/dev-templates?dir=${lang}").devShells.${system}.default;
    in
    {
      devShells = forAllSystems (
        { pkgs }:
        let
          ahshLanguages = builtins.fromJSON (builtins.getEnv "AHSH_LANGUAGES");
          inputsFrom = map (getTemplateShell pkgs.stdenv.hostPlatform.system) ahshLanguages;
        in
        {
          default = pkgs.mkShellNoCC { inherit inputsFrom; };
        }
      );
    };
}
