{
  description = "A devShell example";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    # crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    # flake-utils.inputs.nixpkgs.follows = "nixpkgs";
  };

  # outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
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
          rust-bin.stable."1.90.0".default
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
        ];
      in {
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            # RUST_SRC_PATH =
            #   "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

            shellHook =
              "export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs};";
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
      });
}
