{
  description = "scratch-test-backend";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    scratch-test-koin2627 = {
      url = "github:marcel-scherzinger/scratch-test-koin2627";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    scratch-test-koin2627,
    flake-utils,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in
    flake-utils.lib.eachDefaultSystem (system: rec {
      # devShells.default = import ./shell.nix pkgs;

      buildBackend = {
        exercises ? scratch-test-koin2627,
        cargoHash ? "sha256-5FujgZhRoYaAu1m//aDwlt2iIwhBjiYp5B5VLeUn+1M=",
      }:
        pkgs.rustPlatform.buildRustPackage {
          name = "scratch-test-backend";
          src = ./.;
          buildInputs = [];
          nativeBuildInputs = [pkgs.pkg-config];
          cargoHash = cargoHash;

          postPatch = ''
            echo "Replace url with local files..., if this fails: compare flake.nix with Cargo.toml url"
            grep 'git = "https://github.com/Marcel-Scherzinger/scratch-test-koin2627"' Cargo.toml
            substituteInPlace Cargo.toml \
              --replace 'git = "https://github.com/Marcel-Scherzinger/scratch-test-koin2627"' \
              'path = "${exercises}"'
          '';
        };

      packages.scratch-test-backend = buildBackend {};

      packages.default = self.packages.x86_64-linux.scratch-test-backend;
    });
}
