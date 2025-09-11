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
        # 🔹 This defines the actual package (used for nix build & nix profile install)
        packages.youtube-mpc = pkgs.rustPlatform.buildRustPackage {
          pname = "ytm";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        # 🔹 Alias "default" to the youtube-mpc package
        packages.default = self.packages.${system}.youtube-mpc;

        # Dev shell for hacking on the project
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.mpv
            pkgs.openssl
            pkgs.pkg-config
	    pkgs.clang
            pkgs.atuin
            pkgs.zsh-autosuggestions
            pkgs.zsh-completions
            pkgs.zsh-syntax-highlighting
          ];

          shellHook = ''
            export SHELL=zsh
            exec zsh --login
          '';
        };
      });
}

