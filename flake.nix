{
  description = "Service for simulating keyboard input from Raspberry Pi GPIO";

  nixConfig = {
    extra-substituters = [
      "https://autarch.cachix.org"
    ];
    extra-trusted-public-keys = [
      "autarch.cachix.org-1:/zblMmmXsZmmIKvyq1fgMy7Nqll/abAf+egJpVR0QZI="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    import-tree.url = "github:vic/import-tree";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    flake-parts,
    import-tree,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        (import-tree ./modules)
      ];
    };
}
