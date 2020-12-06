#!/bin/bash

device='/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd'

sudo systemctl stop udevmon.service
sleep 1
echo start

sudo intercept -g "$device" | target/debug/key-mods-rs &

proc_pid=$?

sleep 10

echo stop

sudo killall -9 intercept

sleep 1

sudo systemctl restart udevmon.service
echo restarted udevmon
