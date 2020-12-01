#!/bin/bash

device='/dev/input/by-id/usb-Logitech_G700s_Rechargeable_Gaming_Mouse_017DF9570007-event-mouse'

systemctl stop udevmon.service
sleep 1
echo start

intercept -g "$device" | target/release/key-mods-rs | uinput -d "$device" -c keyboard.yaml &

proc_pid=$?

sleep 10

echo stop

killall -9 intercept

sleep 1

systemctl restart udevmon.service
echo restarted udevmon
