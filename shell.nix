{
  pkgs ? import <nixpkgs> { },
}:

let
  python = with pkgs; python313.withPackages (python-pkgs: with python-pkgs; [ ]);
  lib-path =
    with pkgs;
    pkgs.lib.makeLibraryPath [
      libxkbcommon
      libevdev
      libcap
      udev
      python
    ];

  rust_overlay = import (
    builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"
  );
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rust = pkgs.rust-bin.stable."1.94.1".default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
  };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.libxkbcommon
    rust
  ]
  ++ (with pkgs; [
    libevdev
    udev
  ]);
  nativeBuildInputs = with pkgs; [ pkg-config ];
  packages = with pkgs; [
    automake
    autoconf
    udev
    libevdev
    python
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib-path}"
    export LD_LIBRARY_PATH="${pkgs.hidapi}/lib:$LD_LIBRARY_PATH"

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
