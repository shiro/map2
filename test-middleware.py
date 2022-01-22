import map2

reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd",
])
# mux = map2.Multiplexer(reader)

# writer = map2.Writer(mux)
writer = map2.Writer(reader)

writer.map("pagedown", lambda: map2.exit())

writer.map_key("a", "b")


map2.wait()
