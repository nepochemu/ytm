{
  description = "CLI YouTube MPC-like client in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "youtube-mpc";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.mpv
            pkgs.openssl
            pkgs.pkg-config

            # essentials for your zsh environment
            pkgs.atuin
            pkgs.zsh-autosuggestions
            pkgs.zsh-completions
            pkgs.zsh-syntax-highlighting
          ];

          # drop straight into your Nix-managed zsh setup
          shellHook = ''
            export SHELL=zsh
            exec zsh --login
          '';
        };
      });
}

