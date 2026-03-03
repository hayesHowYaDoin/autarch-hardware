{inputs, ...}: let
  src = inputs.self;

  mkPackage = pkgs:
    pkgs.rustPlatform.buildRustPackage {
      pname = "autarch-hardware";
      version = "0.0.1";
      inherit src;

      cargoLock = {
        lockFile = src + /Cargo.lock;
      };

      meta.mainProgram = "autarch-hardware";
    };
in {
  flake.overlays.default = final: prev: {
    autarch-hardware = mkPackage final;
  };

  perSystem = {
    pkgs,
    lib,
    ...
  }: let
    autarch-hardware = mkPackage pkgs;
    autarch-hardware-aarch64 = mkPackage pkgs.pkgsCross.aarch64-multiplatform;
  in {
    packages = {
      inherit autarch-hardware autarch-hardware-aarch64;
      default = autarch-hardware;
    };

    apps = {
      default = {
        type = "app";
        program = lib.getExe autarch-hardware;
      };
    };
  };
}
