#!/usr/bin/env bash
set -e

version="$1"
shift

if [ -z "$version" ]; then
    echo "usage: release.sh version"
    exit 1
fi

sed -i -re 's/^version = ".*/version = "'"$version"'"/' Cargo.toml
sed -i -re '/^name = "map2"/{n;s/^version = ".*/version = "'"$version"'"/}' Cargo.lock
git reset
git add Cargo.{lock,toml}
git commit . -m "$version"
git push
git tag -a "$version" -m "$version"
git push origin "$version"
