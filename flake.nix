{
  description = "Orbital";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
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
          # Libraries loaded by Bevy via `dlopen()`.
          runtimeDependencies = with pkgs; [
            libxkbcommon
            vulkan-loader
          ];
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo-sweep
              clippy
              lldb # Debugger.
              nil # Nix language server.
              nixfmt
              rustfmt
              vscode
            ];
            inherit buildInputs nativeBuildInputs;
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
          packages.orbital = pkgs.rustPlatform.buildRustPackage {
            pname = "orbital";
            version = "0.1.0";
            src = ./.;
            inherit buildInputs nativeBuildInputs runtimeDependencies;
            # cargoLock = {
            #   lockFile = ./Cargo.lock;
            # };
            cargoHash = "sha256-/ugYSwTO95hEuG08/c5QtfbVBIsnQDfFzsAnFkJz5Xg=";
            # TODO: Install icon with something like:
            # postInstall = ''
            # install -D icon.ico "$out/share/orbital/icon.ico"
            # '';
          };
          packages.default = self'.packages.orbital;
        };
    };
}
