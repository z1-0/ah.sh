{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    flake-root.url = "github:srid/flake-root";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      # See ./nix/modules/*.nix for the modules that are imported here.
      imports = with builtins; map (fn: ./nix/modules/${fn}) (attrNames (readDir ./nix/modules));
    };
}
