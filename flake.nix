{
  description = "Flake templates for easily creating environments by devenv";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    adhoc.url = ./adhoc;
  };

  outputs =
    inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      flake = {
        templates = {
          default = {
            path = ./basic;
            description = "The basic development environment";
          };
          rust = {
            path = ./rust;
            description = "Rust development environment";
          };
        };
      };
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        {
          packages.default = import ./script.nix { inherit pkgs; };

          apps.default = {
            type = "app";
            program = "${config.packages.default}/bin/envshell";
          };

          devShells = {
            default = inputs'.adhoc.devShells.default;
            c = inputs'.adhoc.devShells.c;
            go = inputs'.adhoc.devShells.go;
            haskell = inputs'.adhoc.devShells.haskell;
            java = inputs'.adhoc.devShells.java;
            js = inputs'.adhoc.devShells.js;
            kotlin = inputs'.adhoc.devShells.kotlin;
            lua = inputs'.adhoc.devShells.lua;
            nix = inputs'.adhoc.devShells.nix;
            python = inputs'.adhoc.devShells.python;
            rust = inputs'.adhoc.devShells.rust;
            scala = inputs'.adhoc.devShells.scala;
            ts = inputs'.adhoc.devShells.ts;
          };
        };
    };

}
