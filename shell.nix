{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell { 
    buildInputs = with pkgs; [ libevdev udev libcap ];
    nativeBuildInputs = with pkgs; [ pkg-config libxkbcommon ];
    packages = with pkgs; [
        automake
        autoconf
        automake
    ];

    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
            with pkgs; pkgs.lib.makeLibraryPath [ libevdev udev libcap ]
        }"
    '';
}
