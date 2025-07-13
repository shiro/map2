{ pkgs ? import <nixpkgs> { } }:
    let 
        python = with pkgs; python312.withPackages (python-pkgs: with python-pkgs; [ ]);
        lib-path = with pkgs; pkgs.lib.makeLibraryPath [ libxkbcommon libevdev udev libcap python ];
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
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${ lib-path }"

        # Setup the virtual environment if it doesn't already exist.
        VENV=.venv
        if test ! -d $VENV; then
            virtualenv $VENV
        fi
        source ./$VENV/bin/activate
        export PYTHONPATH=`pwd`/$VENV/${python.sitePackages}/:$PYTHONPATH
    '';
}
