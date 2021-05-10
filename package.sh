#!/bin/bash
filename="map2-$(git describe --abbrev=0 --tags)"
arch="$(uname -m)"

tar -cvzf  "$filename-$arch.tar.gz" -C pkg/map2 --exclude='.[^/]*' .
