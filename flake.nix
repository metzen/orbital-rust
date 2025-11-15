{
  description = "A devShell example";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    #    nixpkgs.config.allowUnfree = true;
    crane.url = "github:ipetkov/crane";
    # crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    # flake-utils.inputs.nixpkgs.follows = "nixpkgs";
  };

  # outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        # overlays = [ ];
        pkgs = import nixpkgs { inherit system overlays; };
        craneLib = crane.mkLib pkgs;
        buildInputs = with pkgs; [
          alsa-lib
          # cargo
          clang
          libxkbcommon
          mold
          nixfmt
          # rustc
          # rustfmt
          systemd
          udev
          # vscode
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          #rust-bin.beta.latest.default
        ];
        nativeBuildInputs = with pkgs; [
          clang
          binutils
          llvmPackages.bintools
          pkg-config
          # TODO: How to override this with extensions in devShell?
          # rust-bin.stable."1.90.0".default
        ];
        rustWithExtensions =
          with pkgs;
          (rust-bin.stable."1.90.0".default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          });
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = nativeBuildInputs ++ [
              rustWithExtensions
            ];
            buildInputs = buildInputs;
            # RUST_SRC_PATH =
            #   "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            RUST_SRC_PATH = "${rustWithExtensions}/lib/rustlib/src";
            packages =
              with (import nixpkgs {
                inherit system;
                config.allowUnfree = true;
              }); [
                vscode
              ];
            shellHook = "export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs};";
          };

        # packages.default = pkgs.rustPlatform.buildRustPackage {
        packages.default = craneLib.buildPackage {
          pname = "orbital";
          version = "0.1.0";
          src = ./.;
          # cargoLock = { lockFile = ./Cargo.lock; };
          cargoArtifacts = craneLib.buildDepsOnly {
            src = ./.;
            strictDeps = true;
            buildInputs = buildInputs;
            nativeBuildInputs = nativeBuildInputs;
          };
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };
      }
    );
}
