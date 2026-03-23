{
  # This is our input set, it contains the channels we will construct the flake from.
  inputs = {
    # Flake-utils allows us to easily support multiple architectures.
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    # Nixpkgs is the main package repo for Nix, we will use it to bring in all of our
    # libraries and tools.
    nixpkgs.url = "github:nixos/nixpkgs";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  # This is the output set, Nix and other Flakes will be able to consume the attributes
  # in the output set.
  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      rust-overlay,
      crane,
      ...
    }@inputs:
    # We wrap the entire output set in this flake-utils function, which builds the flake
    # for each architecture type supported by nix.
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        overlays = [
          rust-overlay.overlays.default
          self.overlays.default
        ];

        # This sets up nixpkgs, where we will pull our dependencies from
        pkgs = (import nixpkgs) {
          # You can insert overlays here by calling `inherit system overlays;`
          inherit overlays;
          localSystem = system;
        };
      in
      {
        packages = {
          downloader = pkgs.downloader.downloader;
          default = pkgs.downloader.downloader;
          dependencies = pkgs.downloader.downloader.cargoArtifacts;
        };

        # This will be entered by direnv, or by manually running `nix shell`. This ensures
        # that our development environment will have all the correct tools at the correct
        # version for this project.
        devShells.default = pkgs.mkShell {
          inputsFrom = [ pkgs.downloader.downloader ];
          inherit (pkgs.downloader.downloader) CLANG_PATH;
          inherit (pkgs.downloader.downloader) LIBCLANG_PATH;
          inherit (pkgs.downloader.downloader) FFMPEG_PATH;
          inherit (pkgs.downloader.downloader) YT_DLP_PATH;
          packages = with pkgs; [
            cargo-hakari
            cargo-nextest
          ];
        };
      }
    )
    // {
      overlays.default = nixpkgs.lib.composeManyExtensions [
        (import ./nix/overlay.nix inputs)
      ];
    };
}
