{
  description = "Fix Spotify shuffling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = [
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.CoreServices
          ];
        };

        spotify-but-with-shuffle-fixed = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in
      {
        checks = {
          inherit spotify-but-with-shuffle-fixed;
        };

        packages.default = spotify-but-with-shuffle-fixed;

        apps.default = flake-utils.lib.mkApp {
          drv = spotify-but-with-shuffle-fixed;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [
          ];
        };
      });
}
