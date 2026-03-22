{
  # crane flake
  crane,
  toolchainOverride ? null,
  # options for override
  example ? null,
  # standard
  pkgs,
  lib,
  # dependencies
  pkg-config,
  openssl,
  git,
  ffmpeg_7-full
}:
let
  nativeBuildInputs = [
    pkg-config
    git
  ];
  buildInputs = [
    openssl
    ffmpeg_7-full
  ];
  toolchain = lib.defaultTo (
    pkgs:
    (pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml).override {
      extensions = [ "rust-src" ];
    }
  ) toolchainOverride;
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
  toolchainNightly = pkgs.rust-bin.nightly.latest.default;
  craneLibNightly = (crane.mkLib pkgs).overrideToolchain toolchainNightly;
  src = lib.cleanSourceWith {
    src = craneLib.path ../.;
    filter =
      let
        data-files = path: _type: builtins.match "^.*\\.(yaml|json|bin|template)$" path != null;
      in
      path: type: (data-files path type) || (craneLib.filterCargoSources path type);
  };
  commonArgsNoDeps = {
    inherit src;
    strictDeps = true;
    inherit nativeBuildInputs buildInputs;
    # cargo check wastes a lot of time, and we run clippy anyway
    cargoCheckCommand = "true";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgsNoDeps;
  commonArgs = commonArgsNoDeps // {
    inherit cargoArtifacts;
  };
in
craneLib.buildPackage (
  commonArgs
  // {
    CLANG_PATH = "${pkgs.clang}/bin/clang";
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    nativeBuildInputs = commonArgs.nativeBuildInputs;
    cargoExtraArgs = if example == null then "--locked" else "--locked --example ${example}";
    pnameSuffix = if example == null then "" else "-${example}";
    fixupPhase = '''';
    # we run tests separately as part of flake checks
    doCheck = false;
  }
)
// {
  inherit craneLib craneLibNightly;
}
