'''
Move the mouse using the 'w', 'a', 's', 'd' directional keys.
'''

import map2
import time
import threading

map2.default(layout="us")

# an easy to use interval utility that allows running functions on a timer
class setInterval:
    def __init__(self, interval, action):
        self.interval = interval / 1000
        self.action = action
        self.stopEvent = threading.Event()
        thread = threading.Thread(target=self.__setInterval)
        thread.start()

    def __setInterval(self):
        nextTime = time.time() + self.interval
        while not self.stopEvent.wait(nextTime - time.time()):
            nextTime += self.interval
            self.action()

    def cancel(self):
        self.stopEvent.set()


# read from keyboard
reader_kbd = map2.Reader(patterns=["/dev/input/by-id/example-keyboard"])

# to move the mouse programmatically, we need a mouse reader we can write into
reader_mouse = map2.Reader()

# add new virtual output devices
writer_kbd = map2.Writer(clone_from="/dev/input/by-id/example-keyboard")
writer_mouse = map2.Writer(capabilities={"rel": True, "buttons": True})

# add mapper
mapper_kbd = map2.Mapper()

# setup the event routing
map2.link([reader_kbd, mapper_kbd, writer_kbd])
map2.link([reader_mouse, writer_mouse])


# we keep a map of intervals that maps each key to the associated interval
intervals = {}


def mouse_ctrl(key, state, axis, multiplier):
    def inner_fn():
        # on key release, remove and cancel the corresponding interval
        if state == 0:
            if key in intervals:
                intervals.pop(key).cancel()
            return

        # this function will move our mouse using the virtual reader
        def send():
            value = 15 * multiplier
            reader_mouse.send("{{relative {} {}}}".format(axis, value))

        # we call it once to move the mouse a bit immediately on key down
        send()
        # and register an interval that will continue to move it on a timer
        intervals[key] = setInterval(20, send)
    return inner_fn


# setup the key mappings
mapper_kbd.map("w up",   mouse_ctrl("w", 0, "Y", -1))
mapper_kbd.map("w down", mouse_ctrl("w", 1, "Y", -1))
mapper_kbd.nop("w repeat")

mapper_kbd.map("a up",   mouse_ctrl("a", 0, "X", -1))
mapper_kbd.map("a down", mouse_ctrl("a", 1, "X", -1))
mapper_kbd.nop("a repeat")

mapper_kbd.map("s up",   mouse_ctrl("s", 0, "Y", 1))
mapper_kbd.map("s down", mouse_ctrl("s", 1, "Y", 1))
mapper_kbd.nop("s repeat")

mapper_kbd.map("d up",   mouse_ctrl("d", 0, "X", 1))
mapper_kbd.map("d down", mouse_ctrl("d", 1, "X", 1))
mapper_kbd.nop("d repeat")


# Keep running forever
map2.wait()
