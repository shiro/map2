import map2

reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-00000300.0-usb-091.0-event-kbd"
])
writer = map2.Writer(reader)

writer.map