#/usr/bin/env bash
set -e

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
export PATH="$HOME/.cargo/bin:$PATH"

rustup target add aarch64-unknown-linux-gnu

apt-get update
apt-get install -y libxkbcommon0 libxkbcommon-dev libxkbcommon-tools automake libtool pkg-config
cat <<EOF >> /etc/apt/sources.list
deb [arch=arm64] http://ports.ubuntu.com/ jammy main multiverse universe
deb [arch=arm64] http://ports.ubuntu.com/ jammy-security main multiverse universe
deb [arch=arm64] http://ports.ubuntu.com/ jammy-backports main multiverse universe
deb [arch=arm64] http://ports.ubuntu.com/ jammy-updates main multiverse universe
EOF
dpkg --print-foreign-architectures
dpkg --add-architecture arm64
dpkg --print-foreign-architectures

# 404 errors here are expected due to missing arm repos
set +e
apt-get update
set -e

apt-get install -y libxkbcommon-dev:arm64
dpkg -L libxkbcommon-dev:arm64
export PATH=~/usr/lib/aarch64-linux-gnu:$PATH

pip install maturin patchelf
