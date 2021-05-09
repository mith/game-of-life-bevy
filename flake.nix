{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    wasm-bindgen-269.url = "github:NixOS/nixpkgs?rev=9bb3fccb5b55326cb3c2c507464a8a28d44d1730";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, naersk, wasm-bindgen-269, ... }:
    utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        rust = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlay
            (self: super: {
              rustc = rust;
              cargo = rust;
              wasm-bindgen-cli = (import wasm-bindgen-269 { inherit system; }).wasm-bindgen-cli;
            })
          ];
        };
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in
      rec {
        packages =
          rec {
            game-of-life = naersk-lib.buildPackage {
              pname = "game-of-life";
              cargoBuildOptions = (old: old ++ [ "--features web" ]);
              CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
              root = ./.;
              nativeBuildInputs = with pkgs; [
                pkg-config
                alsaLib
                libudev
                xorg.libX11
                xlibs.libX11
                xlibs.libXcursor
                xlibs.libXi
                xlibs.libXrandr
                python3
              ];
            };
            game-of-life-web = pkgs.stdenv.mkDerivation {
              name = "game-of-life-web";
              nativeBuildInputs = with pkgs; [
                wasm-bindgen-cli
                game-of-life.overrideAttr (attrs: {
                })
              ];
              phases = [ "installPhase" ];
              installPhase = ''
                mkdir -p $out
                wasm-bindgen --out-dir $out --out-name wasm --target web --no-typescript ${game-of-life}/bin/game-of-life-bevy.wasm
              '';
            };
          };

        defaultPackage = packages.game-of-life;

        apps.game-of-life = utils.lib.mkApp {
          drv = packages.game-of-life;
        };
        defaultApp = apps.game-of-life;

        devShell = pkgs.mkShell {
          inputsFrom = [ packages.game-of-life ];
          RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library/";
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            cargo-edit
            wasm-bindgen-cli
            simple-http-server
            vulkan-loader
            lldb
          ];
        };
      });
}
