{ pkgs ? import <nixpkgs> { } }:
    let python = with pkgs; python312.withPackages (python-pkgs: with python-pkgs; [
        #
    ]);
in
pkgs.mkShell { 
    buildInputs = with pkgs; [ libevdev udev libcap ];
    nativeBuildInputs = with pkgs; [ pkg-config libxkbcommon libevdev udev ];
    packages = with pkgs; [
        automake
        autoconf
        udev
        libevdev
        python
    ];

    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
        with pkgs; pkgs.lib.makeLibraryPath [ libxkbcommon libevdev udev libcap python ]
        }"
    '';
}
