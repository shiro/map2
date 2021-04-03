#!/bin/bash

sudo systemctl stop udevmon.service
sleep 1
echo start

export RUST_BACKTRACE=1

sudo -E udevmon -c udevmon.yaml &

proc_pid=$?

sleep 10

echo stop

sudo killall -9 intercept key-mods-rs udevmon uinput intercept

sleep 1

sudo systemctl restart udevmon.service
echo restarted udevmon
