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

    patchedSources = let
      exerciseRepoInCargoToml = "https://github.com/marcel-scherzinger/scratch-test-koin2627";
      localNameInCargoToml = "skoin2627";
      packageName = "scratch-test-koin2627";
      lock-line-to-remove = ''source = "git+${exerciseRepoInCargoToml}'';
    in
      {exercises}:
        pkgs.stdenv.mkDerivation {
          src = ./.;
          name = "backend-sources";
          buildPhase = let
            text = ''
              [patch."${exerciseRepoInCargoToml}".${localNameInCargoToml}]
              path = "${exercises}"
              package = "${packageName}"
            '';
          in ''
            mkdir $out
            cp -r . $out
            ${pkgs.gnugrep}/bin/grep -i -v '${lock-line-to-remove}' Cargo.lock > $out/Cargo.lock
            echo '${text}' >> $out/Cargo.toml
          '';
        };
    buildBackend = {
      exercises,
      cargoHash,
    }:
      pkgs.rustPlatform.buildRustPackage {
        name = "scratch-test-backend";
        src = "${patchedSources {inherit exercises;}}";
        buildInputs = [];
        nativeBuildInputs = [pkgs.pkg-config];

        inherit cargoHash;
        #cargoLock.lockFile = "${self.packages."${system}".cargoLockWithExPath}";
      };
  in
    {
      inherit buildBackend;
    }
    // flake-utils.lib.eachDefaultSystem (system: rec {
      packages = {
        scratch-test-backend = buildBackend {
          exercises = scratch-test-koin2627;
          cargoHash = "sha256-bm6kK8DhmqeQO/A3ReXYjTZm30FKrFgw3lC8yF0svO0=";
        };

        default = self.packages.${system}.scratch-test-backend;

        scratch-test-backend-sources = patchedSources {exercises = scratch-test-koin2627;};
      };
      # packages.cargoLockWithExPath = pkgs.stdenv.mkDerivation {
      #   src = ./.;
      #   name = "Cargo.lock";
      #   buildPhase = ''
      #     touch $out
      #     ${pkgs.gnugrep}/bin/grep -v '${lock-line-to-remove}' Cargo.lock > $out
      #   '';
      # };
    });
}
