{
  lib,
  openssl,
  pkg-config,
  rustPlatform,
}:
let
  cargoToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "nixpkgs-build-failure-notifier";
  version = cargoToml.package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../src
      ../Cargo.toml
      ../Cargo.lock
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  meta = {
    changelog = "https://github.com/diogotcorreia/nixpkgs-build-failure-notifier/releases/tag/${finalAttrs.version}";
    description = "Get an email when a nixpkgs package fails to build on Hydra";
    homepage = "https://github.com/diogotcorreia/nixpkgs-build-failure-notifier";
    license = lib.licenses.gpl3;
    mainProgram = "nixpkgs-build-failure-notifier";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
})
