#!/usr/bin/python
'''
Maps the keyboard to a virtual controller.

WASD keys -> left joystick
IHJK keys -> right joystick
TFGH keys -> dpad
arrow keys -> A/B/X/Y
q key -> left shoulder
e key -> left shoulder 2
u key -> right shoulder
o key -> right shoulder 2
x key -> left joystick click
m key -> left joystick click
left shift -> select
right shift -> start
spacebar -> exit
'''
import map2

map2.default(layout = "us")

reader = map2.Reader(patterns=["/dev/input/by-id/example-keyboard"])

mapper = map2.Mapper()

controller = map2.Writer(name="virtual-controller", capabilities = {
    "buttons": True,
    "abs": {
        # map joysticks to [0..255]
        "X":     {"value": 128, "min": 0,  "max": 255},
        "Y":     {"value": 128, "min": 0,  "max": 255},
        "RX":    {"value": 128, "min": 0,  "max": 255},
        "RY":    {"value": 128, "min": 0,  "max": 255},
        # map dpad to [-1..1]
        "hat0X": {"value": 0,   "min": -1, "max": 1},
        "hat0Y": {"value": 0,   "min": -1, "max": 1},
    }
})

map2.link([reader, mapper, controller])


# some convenience functions
def joystick(axis, offset):
    def fn():
        # the joystick range is [0..255], so 128 is neutral
        print([axis, offset])
        controller.send("{absolute "+axis+" "+str(128 + offset)+"}")
    return fn

def dpad(axis, offset):
    def fn():
        controller.send("{absolute "+axis+" "+str(offset)+"}")
    return fn

def button(button, state):
    def fn():
        controller.send("{"+button+" "+state+"}")
    return fn


# WASD directional keys to the left joystick
mapper.map("w down", joystick("Y", -80))
mapper.map("w up", joystick("Y", 0))
mapper.nop("w repeat")

mapper.map("a down", joystick("X", -80))
mapper.map("a up", joystick("X", 0))
mapper.nop("a repeat")

mapper.map("s down", joystick("Y", 80))
mapper.map("s up", joystick("Y", 0))
mapper.nop("s repeat")

mapper.map("d down", joystick("X", 80))
mapper.map("d up", joystick("X", 0))
mapper.nop("d repeat")

# map WASD directional keys to the right joystick
mapper.map("i down", joystick("RY", -80))
mapper.map("i up", joystick("RY", 0))
mapper.nop("i repeat")

mapper.map("j down", joystick("RX", -80))
mapper.map("j up", joystick("RX", 0))
mapper.nop("j repeat")

mapper.map("k down", joystick("RY", 80))
mapper.map("k up", joystick("RY", 0))
mapper.nop("k repeat")

mapper.map("l down", joystick("RX", 80))
mapper.map("l up", joystick("RX", 0))
mapper.nop("l repeat")

# TFGH directional keys to the left joystick
mapper.map("t down", dpad("hat0Y", -1))
mapper.map("t up", dpad("hat0Y", 0))
mapper.nop("t repeat")

mapper.map("f down", dpad("hat0X", -1))
mapper.map("f up", dpad("hat0x", 0))
mapper.nop("f repeat")

mapper.map("g down", dpad("hat0Y", 1))
mapper.map("g up", dpad("hat0Y", 0))
mapper.nop("g repeat")

mapper.map("h down", dpad("hat0X", 1))
mapper.map("h up", dpad("hat0X", 0))
mapper.nop("h repeat")

# A/B/X/Y buttons (or whatever other naming)
mapper.map("up",    "{btn_north}")
mapper.map("down",  "{btn_south}")
mapper.map("left",  "{btn_west}")
mapper.map("right", "{btn_east}")

# left shoulder buttons
mapper.map("q", "{btn_tl}")
mapper.map("e", "{btn_tl2}")

# right shoulder buttons
mapper.map("u", "{btn_tr}")
mapper.map("o", "{btn_tr2}")

# start/select buttons
mapper.map("left_shift",  "{btn_select}")
mapper.map("right_shift", "{btn_start}")

# joystick buttons
mapper.map("x", "{btn_thumbl}")
mapper.map("m", "{btn_thumbr}")

# exit wtih space
mapper.map("space", lambda: map2.exit())


# keep running
map2.wait()
