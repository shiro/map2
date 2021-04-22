#!/bin/bash


sudo systemctl stop udevmon.service


target/debug/dev-test /dev/input/by-id/usb-Logitech_USB_Receiver-if01-event-mouse &

proc_pid=$!

sleep 5

sudo kill $proc_pid

sleep 1

sudo systemctl restart udevmon.service



