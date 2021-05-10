#!/bin/bash
filename="map2-$(git describe --abbrev=0 --tags)"

tar -cvzf  "$filename.tar.gz" -C pkg/map2 --exclude='.[^/]*' .
