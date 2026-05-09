{
  description = "scratch-test-backend";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    # devShells."x86_64-linux".default = import ./shell.nix pkgs;

    packages."x86_64-linux".scratch-test-backend = pkgs.rustPlatform.buildRustPackage {
      name = "scratch-test-backend";
      src = ./.;
      buildInputs = [];
      nativeBuildInputs = [pkgs.pkg-config];
      cargoHash = "sha256-v1ZJ536DIkWf5lQENxfu6EDaZqRdd2FRNLNrT5taQ0M=";
    };

    packages.x86_64-linux.default = self.packages.x86_64-linux.scratch-test-backend;
  };
}

