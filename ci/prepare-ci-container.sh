#!/bin/bash

set -e

if command -v apt-get &> /dev/null; then
    echo "using apt-get"
    apt-get update
    apt-get install -y libxkbcommon0 libxkbcommon-dev libxkbcommon-tools automake libtool pkg-config

    cat <<EOF >> /etc/apt/sources.list
    deb [arch=arm64] http://ports.ubuntu.com/ jammy main multiverse universe
    deb [arch=arm64] http://ports.ubuntu.com/ jammy-security main multiverse universe
    deb [arch=arm64] http://ports.ubuntu.com/ jammy-backports main multiverse universe
    deb [arch=arm64] http://ports.ubuntu.com/ jammy-updates main multiverse universe
EOF
    dpkg --add-architecture arm64
    set +e
    apt-get update
    set -e
    apt-get install -y libxkbcommon-dev:arm64
    dpkg -L libxkbcommon-dev:arm64
    export PATH=~/usr/lib/aarch64-linux-gnu:$PATH
    export RUSTFLAGS='-L /usr/lib/aarch64-linux-gnu'
    # hack for maturin wheel repair not picking up rust flags
    # https://github.com/PyO3/maturin/discussions/2092#discussioncomment-9648400
    cp /usr/lib/aarch64-linux-gnu/libxkbcommon.so.0.0.0 /usr/aarch64-unknown-linux-gnu/aarch64-unknown-linux-gnu/sysroot/lib64/libxkbcommon.so
    cp /usr/lib/aarch64-linux-gnu/libxkbcommon.so.0.0.0 /usr/aarch64-unknown-linux-gnu/aarch64-unknown-linux-gnu/sysroot/lib64/libxkbcommon.so.0
elif command -v yum &> /dev/null; then
    echo "using yum"
    yum install -y libxkbcommon-devel libatomic

    # build pkg-config manually due to a bug in the old version from the repo
    cd /tmp
    git clone https://github.com/pkgconf/pkgconf
    cd pkgconf
    ./autogen.sh
    ./configure \
         --with-system-libdir=/lib:/usr/lib \
         --with-system-includedir=/usr/include
    make
    make install
fi
