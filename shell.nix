{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell { 
    buildInputs = with pkgs; [ libevdev ];
    nativeBuildInputs = with pkgs; [ pkg-config libxkbcommon ];
    packages = with pkgs; [
        automake
        autoconf
        automake
    ];

    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
            pkgs.lib.makeLibraryPath [ pkgs.libevdev ]
        }"
    '';
}
