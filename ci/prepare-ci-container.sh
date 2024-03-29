#!/bin/bash

set -e


yum install -y libxkbcommon-devel libatomic

cd /tmp
git clone https://github.com/pkgconf/pkgconf
cd pkgconf

./autogen.sh
./configure \
     --with-system-libdir=/lib:/usr/lib \
     --with-system-includedir=/usr/include
make
make install
