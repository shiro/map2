#!/bin/bash

sleep 1

sudo systemctl stop udevmon.service


target/debug/key-mods-rs example.km  &

proc_pid=$!

sleep $1

sudo kill $proc_pid

sleep 1

sudo systemctl restart udevmon.service



