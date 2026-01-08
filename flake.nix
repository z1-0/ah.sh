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
    {
      self,
      nixpkgs,
      devenv,
      ...
    }@inputs:
    let
      inherit (nixpkgs) lib;

      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f:
        lib.genAttrs allSystems (
          system:
          f {
            pkgs = import nixpkgs { inherit system; };
          }
        );

      ahshLanguages = builtins.fromJSON (builtins.getEnv "AHSH_LANGUAGES");
      ahshPackages = builtins.fromJSON (builtins.getEnv "AHSH_PACKAGES");
      cargoToml = builtins.fromTOML (builtins.readFile (self + /Cargo.toml));
    in
    {
      apps = forAllSystems (
        { pkgs }:
        {
          default = {
            type = "app";
            program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/ah";
          };
        }
      );

      packages = forAllSystems (
        { pkgs }:
        {
          ah = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            AHSH_NIXPKGS_SRC = builtins.toString nixpkgs;
            AHSH_DEVENV_SRC = builtins.toString devenv;
          };
          default = self.packages.${pkgs.stdenv.hostPlatform.system}.ah;
        }
      );

      devShells = forAllSystems (
        { pkgs }:
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {
                languages.nix.enable = true;
                languages.rust.enable = true;
                packages = [
                  pkgs.nixfmt
                  pkgs.rustfmt
                  pkgs.prettier
                ];
              }
            ];
          };

          ah = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              (lib.optionalAttrs (ahshLanguages != [ ]) {
                languages = lib.genAttrs ahshLanguages (_: {
                  enable = true;
                });
              })
              (lib.optionalAttrs (ahshPackages != [ ]) {
                packages = builtins.map (package: pkgs.${package}) ahshPackages;
              })
            ];
          };
        }
      );
    };
}
