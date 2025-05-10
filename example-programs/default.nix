let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.11";
  pkgs = import nixpkgs { config = {}; overlays = []; };
  mkCApplication = name: pkgs.stdenv.mkDerivation {
    name = name;
    src = ./${name};
    buildPhase = ''
      $CC ${name}.c -o ${name}
    '';
    installPhase = ''
      mkdir -p $out/bin
      cp ${name} $out/bin/${name}
    '';
  };
in
{
  multiple_prints = mkCApplication "multiple_prints";
}