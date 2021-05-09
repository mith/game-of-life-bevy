{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    wasm-bindgen-legacy.url = "github:NixOS/nixpkgs?rev=9bb3fccb5b55326cb3c2c507464a8a28d44d1730";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, naersk, wasm-bindgen-legacy, ... }:
    utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        rust = pkgs.rust-bin.stable.latest.default.override {
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
              wasm-bindgen-cli = (import wasm-bindgen-legacy { inherit system; }).wasm-bindgen-cli;
            })
          ];
        };
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in
      rec {
        packages.game-of-life = naersk-lib.buildPackage {
          pname = "game-of-life";
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
        defaultPackage = packages.game-of-life;

        apps.game-of-life = utils.lib.mkApp {
          drv = packages.game-of-life;
        };
        defaultApp = apps.game-of-life;

        devShell = pkgs.mkShell {
          inputsFrom = [ packages.game-of-life ];
          RUST_SRC_PATH="${pkgs.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library/";
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
