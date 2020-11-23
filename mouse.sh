#!/bin/bash

device='/dev/input/by-path/pci-0000:03:00.0-usb-0:4:1.1-event-mouse'

sudo intercept -g "$device" | target/debug/key-mods-rs | sudo uinput -d "$device" &
# sudo intercept -g "$device" | sudo uinput -d "$device" &
# sudo intercept -g "$device" | ./key-mods-rs &
# sudo intercept -g "$device" &

proc_pid=$?

sleep 15

sudo kill $proc_pid
sudo killall -9 intercept

exit 0
