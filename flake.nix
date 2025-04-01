{
  description = "a minimal debugger written in Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    typst-packages = {
      url = "github:typst/packages";
      flake = false;
    };
    typix = {
      url = "github:loqusion/typix";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {self, nixpkgs, devshell, utils, ...}@inputs:
    utils.lib.eachDefaultSystem (system:
      let
        lib = nixpkgs.lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };
        typstPackagesCache = pkgs.stdenv.mkDerivation {
          name = "typst-packages-cache";
          src = inputs.typst-packages;
          dontBuild = true;
          installPhase = ''
            mkdir -p "$out/typst/packages"
            cp --dereference --no-preserve=mode --recursive --reflink=auto \
              --target-directory="$out/typst/packages" -- "$src"/packages/*
          '';
        };

        rust-target = pkgs.pkgsStatic.targetPlatform.rust.rustcTarget;
        fenix = inputs.fenix.packages.${system};
        rust-toolchain = with fenix; combine [
              latest.rustc
              latest.cargo
              latest.clippy
              latest.rustfmt
              targets.${rust-target}.latest.rust-std
        ];
      in
      {
        packages.debugger-rs-paper = inputs.typix.lib.${system}.buildTypstProject {
          name = "debugger-rs-paper.pdf";
          src = ./work/main.typ;
          XDG_CACHE_HOME = typstPackagesCache;
        };

        git.hooks = {
          enable = true;
          pre-commit.text = "nix flake check";
        };

        devShells.default = (pkgs.devshell.mkShell {
          name = "debugger-rs";
          packages = with pkgs; [
            stdenv.cc
            coreutils

            rust-toolchain
            rust-analyzer
            cargo-expand
            cargo-watch

            typst
            tinymist
          ];
        
          commands = [
            {
              name = "watch-paper";
              command = ''
                typst watch --root "$PRJ_ROOT/paper" "$PRJ_ROOT/paper/main.typ"
              '';
              help = "watch and recompile the typst paper";
            }
            {
              name = "watch-app";
              command = ''
                cargo watch --workdir ./application
              '';
              help = "watch and recompile the main application";
            }
          ];
        });
      });
  # });
}
