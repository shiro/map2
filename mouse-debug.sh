#!/bin/bash

# /dev/input/by-path
# pci-0000:03:00.0-usb-0:4:1.1-event-mouse
# pci-0000:03:00.0-usb-0:4:1.1-mouse
# pci-0000:03:00.0-usb-0:8:1.0-event
# pci-0000:03:00.0-usb-0:9:1.0-event-kbd
# pci-0000:03:00.0-usb-0:9:1.1-event
# pci-0000:03:00.0-usb-0:9:1.2-event-mouse
# pci-0000:03:00.0-usb-0:9:1.2-mouse
# pci-0000:11:00.3-usb-0:1:1.0-event-joystick
# pci-0000:11:00.3-usb-0:1:1.0-joystick
# platform-AMDI0010:03-event
# platform-AMDI0010:03-event-mouse
# platform-AMDI0010:03-mouse
# platform-pcspkr-event-spkr

device='/dev/input/by-path/pci-0000:03:00.0-usb-0:4:1.1-event-mouse'

sudo intercept -g "$device" | target/debug/key-mods-rs &

proc_pid=$?

sleep 8

sudo kill $proc_pid
sudo killall -9 intercept

exit 0
