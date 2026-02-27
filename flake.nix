{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/DeterminateSystems/nixpkgs-weekly/0.1";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.git-hooks-nix.flakeModule
      ];

      systems = import inputs.systems;

      perSystem =
        {
          config,
          system,
          pkgs,
          lib,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          packages =
            let
              cargoToml = fromTOML (builtins.readFile (self + /Cargo.toml));
            in
            {
              ah = pkgs.rustPlatform.buildRustPackage {
                inherit (cargoToml.package) version;
                pname = cargoToml.package.name;
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
              };
              default = self.packages.${system}.ah;
            };

          apps.default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/ah";
          };

          devShells.default = pkgs.mkShellNoCC {
            inherit (config.pre-commit.devShell) shellHook;

            packages = [
              pkgs.nixd
              pkgs.rust-bin.stable.latest.complete
            ]
            ++ (lib.attrValues config.treefmt.build.programs)
            ++ config.pre-commit.settings.enabledPackages;
          };

          pre-commit.settings.hooks = {
            treefmt = {
              enable = true;
              package = config.treefmt.build.wrapper;
            };
            clippy = {
              enable = true;
              package = pkgs.rust-bin.stable.latest.clippy;
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
                "flake-checker"
                "markdownlint"
                "mixed-line-endings"
                "statix"
                "trim-trailing-whitespace"
                "typos"
              ]
              (_: {
                enable = true;
              });

          treefmt.programs = {
            nixfmt.enable = true;
            prettier.enable = true;
            rustfmt = {
              enable = true;
              package = pkgs.rust-bin.stable.latest.rustfmt;
            };
          };
        };
    };
}
