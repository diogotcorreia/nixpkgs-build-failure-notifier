{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          nixpkgs-build-failure-notifier = pkgs.callPackage ./nix/package.nix { };
          default = nixpkgs-build-failure-notifier;
        }
      );

      devShell = eachSystem (
        system:
        import ./shell.nix {
          pkgs = import nixpkgs { inherit system; };
        }
      );

      nixosModules = rec {
        nixpkgs-build-failure-notifier = import ./nix/module.nix;
        default = nixpkgs-build-failure-notifier;
      };
    };
}
