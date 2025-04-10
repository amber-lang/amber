{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixpkgs-unstable";

    naersk = {
      url = "github:nix-community/naersk?ref=master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , naersk
    , rust-overlay
    ,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naersk-lib = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages.default = naersk-lib.buildPackage {
          src = ./.;
          nativeBuildInputs = [ pkgs.makeWrapper pkgs.cargo-edit ];
          postConfigure = ''
            cargo set-version "$version-nix-${self.shortRev or "dirty"}"
          '';
          postInstall = ''
            wrapProgram "$out/bin/amber" --prefix PATH : ${nixpkgs.lib.makeBinPath [ pkgs.bc ]}
          '';
        };
        devShells.default =
          pkgs.mkShell {
            packages = with pkgs; [
              bc
              pre-commit
            ] ++ [ toolchain ];
            RUST_SRC_PATH = toolchain.availableComponents.rust-src;
          };
      }
    );
}
