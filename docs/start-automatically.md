# Start automatically

A common use case for map2 scripts is to run all the time, so starting them
automatically in the background on startup / login makes a lot of sense.  
There are several methods to do this, most of which are described in detail on
[this Arch Wiki page](https://wiki.archlinux.org/title/Autostarting).

## Systemd

If systemd is installed on the system, it is possible to start scripts on login
by creating a new unit file:

*~/.config/systemd/user/map2.service:*

```
[Unit]
Description=map2 script

[Service]
Type=exec
ExecStart=map2 -d /path/to/device.list /path/to/script.m2

[Install]
WantedBy=multi-user.target
```

And running a few simple commands:

```
$ systemctl --user daemon-reload
$ systemctl --user enable map2
$ systemctl --user start map2
```
