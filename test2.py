import map2

reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"
])

writer = map2.Writer(reader)


def hi():
    print("hi")


writer.map("a", "c")
writer.map("s", hi)

writer.map("q", lambda: map2.exit())

map2.wait()
