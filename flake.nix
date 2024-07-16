{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      nixpkgs-mozilla,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import nixpkgs-mozilla) ];
        };
        toolchain =
          (pkgs.rustChannelOf {
            rustToolchain = ./rust-toolchain.toml;
            sha256 = "sha256-Ngiz76YP4HTY75GGdH2P+APE/DEIx2R/Dn+BwwOyzZU=";
          }).rust;
        naersk-lib = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      {
        packages.default = naersk-lib.buildPackage {
          src = ./.;
          postInstall = ''
            wrapProgram "$out/bin/amber" --set PATH ${nixpkgs.lib.makeBinPath [ pkgs.bc ]}
          '';
        };
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              bc
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ];
            nativeBuildInputs = [ toolchain ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
