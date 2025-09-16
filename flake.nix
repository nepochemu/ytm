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
        packages.youtube-mpc = pkgs.rustPlatform.buildRustPackage {
          pname = "ytm";
          version = "0.1.0";
          src = ./.;
        #  cargoLock = {
        #    lockFile = ./Cargo.lock;
        #  };
        
	cargoHash = "sha256-9bG56/rIp/oMB4GRi6Pp5aMjbSW5hMQAahYqo8cUeWE=" s;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        packages.default = self.packages.${system}.youtube-mpc;

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

