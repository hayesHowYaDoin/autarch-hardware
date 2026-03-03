{inputs, ...}: {
  perSystem = {
    pkgs,
    lib,
    ...
  }: let
    rpkgs = import inputs.nixpkgs {
      inherit (pkgs) system;
      overlays = [inputs.rust-overlay.overlays.default];
    };
  in {
    devShells.default = rpkgs.mkShell rec {
      nativeBuildInputs = with rpkgs; [
        pkg-config
      ];

      buildInputs = with rpkgs; [
        bacon
        just
        nixd
        (rust-bin.stable."1.93.1".default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = [
            "aarch64-unknown-linux-gnu"
            "x86_64-unknown-linux-gnu"
          ];
        })
      ];

      LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
      AUTARCH_HARDWARE_LOG_PATH = ".";
    };
  };
}
