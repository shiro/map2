#!/usr/bin/env bash

set -eux

HEADER_DIR=evdev-sys/libevdev/include/linux/linux

./tools/make-event-names.py $HEADER_DIR/input-event-codes.h $HEADER_DIR/input.h | head -n -1 > src/enums.rs
