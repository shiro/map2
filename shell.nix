{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell { 
    buildInputs = with pkgs; [ libevdev udev libcap ];
    nativeBuildInputs = with pkgs; [ pkg-config libxkbcommon libevdev udev ];
    packages = with pkgs; [
        automake
        autoconf
        automake
        udev
        libevdev
    ];

    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
        with pkgs; pkgs.lib.makeLibraryPath [ libxkbcommon libevdev udev libcap ]
        }"
    '';
}
