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

          cargoHash = "sha256-Pv+1iMwhyobgQUMTgxaxm0YKZbjfaEsIF0oH3BYpRKA=";

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.makeWrapper
          ];
          buildInputs = [
            pkgs.openssl
            pkgs.mpv
            pkgs.cacert
            pkgs.rust-analyzer
            pkgs.fzf
          ];

          postInstall = ''
           wrapProgram $out/bin/ytm \
           --set PATH ${pkgs.lib.makeBinPath [ pkgs.mpv pkgs.cacert pkgs.fzf ]}
          '';

        };

        packages.default = self.packages.${system}.youtube-mpc;

        apps.default = pkgs.buildFHSEnv {
          name = "ytm-fhs";
          targetPkgs = pkgs: [ pkgs.openssl pkgs.cacert pkgs.mpv ];
          runScript = "${self.packages.${system}.youtube-mpc}/bin/ytm";
        };

        apps.ytm-fhs = {
          type = "app";
          program =
            let fhsEnv = pkgs.buildFHSEnv {
              name = "ytm-fhs";
              targetPkgs = pkgs: [ pkgs.openssl pkgs.cacert pkgs.mpv ];
              runScript = "${self.packages.${system}.youtube-mpc}/bin/ytm";
            };
            in "${fhsEnv}/bin/ytm-fhs";
        };

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
            pkgs.cacert
            pkgs.fzf
            pkgs.rustPlatform.rustLibSrc
          ];

          # 🔑 Tell Cargo where openssl.pc lives
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          shellHook = ''
  export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
  export SHELL=zsh
  exec zsh --login
'';

        };
      });
}