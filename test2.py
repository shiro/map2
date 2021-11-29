import map2

reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd",
])
writer = map2.Writer(reader)

writer.map("pagedown", lambda: map2.exit())

writer.map_key("{a down}", "b")
writer.map_key("{a repeat}", "c")
writer.map_key("{a up}", "d")


map2.wait()
