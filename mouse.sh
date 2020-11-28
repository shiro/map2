#!/bin/bash

device='/dev/input/by-id/usb-Logitech_G700s_Rechargeable_Gaming_Mouse_017DF9570007-event-mouse'

sudo systemctl stop udevmon.service
sleep 1
echo start

sudo intercept -g "$device" | target/debug/key-mods-rs | sudo uinput -d "$device" -c keyboard.yaml &

proc_pid=$?

sleep 10

echo stop

sudo killall -9 intercept

sleep 1

sudo systemctl restart udevmon.service
echo restarted udevmon
