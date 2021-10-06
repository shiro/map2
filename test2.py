import map2
import time

reader = map2.EventReader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"
])
writer = map2.EventWriter(reader)


def hi():
    print("hi")


writer.map("a", "c")
writer.map("s", hi)

time.sleep(5)
