# Map2
 
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A python library that allows complex key remapping on Linux, written in
Rust.

**Warning:** this is the experimental python branch, the API is likely to
change, please don't be annoyed or wait until it's on master.

All of the functionality related to interacting with graphical elements such as
getting the active window information is currently only supported on X11.
Wayland support is planned but probably won't be added for some time.

For details check the [documentation](#documentation).

<ins>Important note:</ins>
The legacy version of map2 which used a custom scripting langauge is located
[here](https://github.com/shiro/map2-legacy).  
Due to performance and growing requirements map2 now uses python for the user
scripts. Map2 itself is still written in rust and compiled for maximum
performance.

# Examples

Map2 supports use cases ranging all the way from simple key remappings to
using complex callbacks and variables.

```.py
import map2

# a reader intercepts all input from device file descriptiors
reader = map2.Reader(patterns=[
    "/dev/input/by-id/usb-Logitech_USB_Receiver-if01-event-.*",
    "/dev/input/by-id/usb-Logitech_G700s_Rechargeable_Gaming_Mouse_017DF9570007-.*-event-.*",
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd",
])

# a writer is responsible for remapping and outputting the modified input event stream
# it attaches to a reader, meaning the event stream is piped through
writer = map2.Writer(reader)

# change the 'a' key to 'b'
writer.map_key("a", "b")

def hello_world():
  print("hello world")
  
  # type the text using the virtual keyboard
  writer.send("some text{enter}")

# map the 'c' key to a callback function
writer.map("c", hello_world)

# a window object listens to changes in the currently active window
window = map2.Window()

# do something on window change
def on_window_change(active_window_class):
  if active_window_class == "firefox"):
    print("firefox is now the active window")
    
    # map 'F1' to ctrl+'t' (open new browser tab)
    writer.map_key("f1", "^t")
  else:
    print("firefox is not the active window")
    
    # map 'F1' back
    writer.map_key("f1", "f1")
  }

# register the callback
window.on_window_change(on_window_change)

# keep running until terminated
map2.wait()
```

For more examples check the [examples directory](examples/README.md).

# Getting started

**tldr:**

- find the input device file descriptors you want to grab
- write a script file for remapping the keys, pass the descriptors to a
  new `reader` object, remap keys on a new `writer` object
- run `$ python my_remapping_script.py`


**Detailed documentation is still being worked on, consult the example scripts.**


## Install

TODO this section

- add `etc/udev/rules.d/10-my-udev.rules`

```
groupadd map2

# create "map2" group
KERNEL=="event*", SUBSYSTEM=="input", ATTRS{hid}=="MYDEV000", MODE="0644", GROUP="map2", SYMLINK+="input/mydevice"

# add yourself to the "map2" group
$ usermod -aG `whoami` map2
```



### Arch Linux

**Arch packages are not available yet for the python branch.**

### Other distributions

Build local package from cloned source:

- download the source code from this repository
- install [maturin](https://github.com/PyO3/maturin) and rust
- build using `$ maturin build --release`
- install using `$ pip install target/wheels/NAME_OF_WHEEL_FILE.whl`

# Documentation

- [start automatically on startup/login](docs/start-automatically.md)

**Detailed documentation is still being worked on, consult the example scripts.**

# Feature roadmap

- [ ] finalize python branch
- [ ] documentation (async callbacks, public API, etc.)
- [ ] deploy python branch to package repos
- [ ] fd patterns file
- [ ] add tests
- [ ] AHK-style hotstrings
- [ ] escaped characters in strings and key sequences
- [ ] pre-packaged binaries for various distros
- [ ] mouse events
- [ ] Wayland support (someday)

# Contributing

If you want to report bugs, add suggestions or help out with development please
check the [issues page](https://github.com/shiro/map2/issues) and open an issue
if it doesn't exist yet.

# License

MIT

# Authors

- shiro <shiro@usagi.io>
