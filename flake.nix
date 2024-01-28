{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
  let
    pversion = "v0.5.14";
    systemBuildInputs = pkgs: {
      x86_64-linux = [
        pkgs.iconv
      ];
      x86_64-darwin =  [
        pkgs.iconv
        pkgs.darwin.apple_sdk.frameworks.Security
        pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      ];
    };

  in
  utils.lib.eachSystem [ "x86_64-darwin" "x86_64-linux" ] (system:
  let
    pkgs = import nixpkgs { inherit system; };
    naersk-lib = pkgs.callPackage naersk { };

    src  =  pkgs.fetchzip {
      name = "src";
      url = "https://github.com/zurawiki/gptcommit/archive/refs/tags/${pversion}.tar.gz";
      hash = "sha256-xjaFr1y2Fd7IWbJlegnIsfS5/oMJYd6QTnwp7IK17xM=";
    };
  in
  {
    defaultPackage = naersk-lib.buildPackage  {
      inherit src;
      buildInputs = (systemBuildInputs pkgs).${system};
    };

    devShell = with pkgs; mkShell {
      buildInputs = [ 
        cargo rustc rustfmt pre-commit rustPackages.clippy 
      ] ++ (systemBuildInputs pkgs).${system};
      RUST_SRC_PATH = rustPlatform.rustLibSrc;
    };
  });
}
