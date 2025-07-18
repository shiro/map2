{ pkgs ? import <nixpkgs> { } }:

let 
    python = with pkgs; python312.withPackages (python-pkgs: with python-pkgs; []);
    lib-path = with pkgs; pkgs.lib.makeLibraryPath [ libxkbcommon libevdev udev libcap python ];

    rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
    pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
    rust = pkgs.rust-bin.nightly."2025-03-10".default.override {
        extensions = [ "rust-src" ];
    };
in
pkgs.mkShell { 
    buildInputs = [
      rust
    ] ++ (with pkgs; [
       libevdev udev libcap 
    ]);
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
        VENV=venv
        if test ! -d $VENV; then
            python -m venv $VENV
            source ./$VENV/bin/activate
            pip install -r requirements.txt
        else
            source ./$VENV/bin/activate
        fi
    '';
    RUST_BACKTRACE = 1;
}
