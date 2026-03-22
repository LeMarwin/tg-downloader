{
  self,
  crane,
  ...
}:
final: prev:
let
  pkgs = final;
in
{
  downloader = (prev.downloader or (final.lib.makeScope final.newScope (_: { }))).overrideScope (
    final: prev:
    pkgs.lib.recursiveUpdate prev {
      downloader = final.callPackage ./package.nix {
        inherit crane;
      };
    }
  );
}
