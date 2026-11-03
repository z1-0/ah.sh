{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    { self, ... }@inputs:

    with inputs;

    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;

      imports = [
        treefmt-nix.flakeModule
        git-hooks-nix.flakeModule
      ];

      perSystem =
        {
          config,
          system,
          lib,
          ...
        }:
        let
          pkgs = import nixpkgs { inherit system; };

          craneLib = crane.mkLib pkgs;

          src = lib.cleanSourceWith {
            filter =
              path: type: (builtins.match ".*assets/.*" path) != null || craneLib.filterCargoSources path type;
            src = ./.;
          };

          commonArgs = {
            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          ah = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
              nativeBuildInputs = [ pkgs.installShellFiles ];
              postInstall = ''
                export HOME=$(mktemp -d)
                installShellCompletion --cmd ah \
                  --bash <($out/bin/ah completion bash) \
                  --zsh <($out/bin/ah completion zsh) \
                  --fish <($out/bin/ah completion fish)
              '';
            }
          );
        in
        {
          checks = {
            inherit ah;
            cargoClippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );
            cargoAudit = craneLib.cargoAudit { inherit src advisory-db; };
            cargoNextest = craneLib.cargoNextest (commonArgs // { inherit cargoArtifacts; });
          };

          packages = {
            inherit ah;
            default = ah;
          };

          apps.default = {
            type = "app";
            program = lib.getExe' ah "ah";
            meta.description = "ah.sh - Ad-hoc development shell manager powered by Nix";
          };

          devShells.default = craneLib.devShell {
            inherit (config.pre-commit.devShell) shellHook;
            checks = self.checks.${system};
            packages =
              (lib.attrValues config.treefmt.build.programs)
              ++ config.pre-commit.settings.enabledPackages
              ++ [
                pkgs.rust-analyzer
              ];
          };

          pre-commit.settings.hooks = {
            treefmt = {
              enable = true;
              package = config.treefmt.build.wrapper;
            };
          }
          //
            lib.genAttrs
              [
                "check-json"
                "check-toml"
                "check-xml"
                "check-yaml"
                "editorconfig-checker"
                "fix-byte-order-marker"
                "mixed-line-endings"
                "trim-trailing-whitespace"
              ]
              (_: {
                enable = true;
              });

          treefmt = {
            programs = {
              nixfmt.enable = true;
              prettier.enable = true;
              taplo.enable = true;
              rustfmt = {
                enable = true;
                package = pkgs.rustfmt;
              };
            };
            settings.excludes = [
              "src/assets/config.schema.json"
              "CHANGELOG.md"
            ];
          };
        };
    };
}
