{ pkgs ?
  # Default to a pinned version of Nixpkgs. The actual revision of the Nixpkgs
  # repository is stored in a separate file (as a fetchTarball Nix expression).
  # We then fetch that revision from Github and import it. The revision should
  # periodically be updated to be the last commit of Nixpkgs.
  import (import ./nixpkgs-pinned.nix) {}
}:

with pkgs;

let
  # Define the unpack phase manually, because setting src = ./. includes too
  # much, and setting up filters is tedious.
  sources = stdenv.mkDerivation {
    name = "tako-src";
    phases = [ "unpackPhase" ];
    unpackPhase = ''
      mkdir $out
      cp ${./Cargo.toml} $out/Cargo.toml
      cp ${./Cargo.lock} $out/Cargo.lock
      cp -r ${./src} $out/src
    '';
  };
in
  rustPlatform.buildRustPackage rec {
    name = "tako-${version}";
    version = "0.0.0";
    src = sources;
    cargoSha256 = "1f7n67xjv268ciw434bicnmc341lgwa0is80r61p6hx9jfn0rjp3";
    nativeBuildInputs = [ curl libsodium pkgconfig ];
    meta = with stdenv.lib; {
      description = "Updater for single files.";
      homepage = https://github.com/ruuda/tako;
      license = licenses.asl20;
      maintainers = [ maintainers.ruuda ];
      platforms = platforms.linux;
    };
  }
