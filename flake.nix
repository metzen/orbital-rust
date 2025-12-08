{
  description = "Orbital";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        buildInputs = with pkgs; [
          alsa-lib
          systemd
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        nativeBuildInputs = with pkgs; [
          autoPatchelfHook
          binutils
          cargo
          clang
          llvmPackages.bintools
          mold
          pkg-config
          rustc
        ];
        runtimeDependencies = with pkgs; [
          libxkbcommon
          vulkan-loader
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            with (import nixpkgs {
              inherit system;
              config.allowUnfree = true;
            }); [
              clippy
              lldb # Debugger.
              nil
              nixfmt
              rustfmt
              vscode
            ];
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}";
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
        packages.orbital = pkgs.rustPlatform.buildRustPackage {
          pname = "orbital";
          version = "0.1.0";
          src = ./.;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
          runtimeDependencies = runtimeDependencies;
          # cargoLock = {
          #   lockFile = ./Cargo.lock;
          # };
          cargoHash = "sha256-/ugYSwTO95hEuG08/c5QtfbVBIsnQDfFzsAnFkJz5Xg=";
          buildType = "debug";
        };
        defaultPackage = self.packages.${system}.orbital;
      }
    );
}
