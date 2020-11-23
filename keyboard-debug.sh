#!/bin/bash

device='/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd'

sudo intercept -g "$device" | target/debug/key-mods-rs &

proc_pid=$?

sleep 8

sudo kill $proc_pid
sudo killall -9 intercept

exit 0
