{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/DeterminateSystems/nixpkgs-weekly/0.1";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
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
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          }
        );

      cargoToml = fromTOML (builtins.readFile (self + /Cargo.toml));
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
          };
          default = self.packages.${pkgs.stdenv.hostPlatform.system}.ah;
        }
      );

      devShells = forAllSystems (
        { pkgs }:
        {
          default =
            with pkgs;
            mkShellNoCC {
              packages = [
                nixd
                nixfmt
                statix
                rust-bin.stable.latest.default
                rust-analyzer
                rustfmt
              ];
            };
        }
      );
    };
}
