import asyncio

import map2
import time


# def hi(a):
#     print("got key: '{}' with value '{}'".format(a.code, a.value))
#
# handle = map2.setup(hi)
#
# print("handle obtained: {}".format(handle))
#
# handle.map("a", "c")
# handle.map("+a", "b")
# handle.map("d", "{a down}")
#
# handle.map("{l down}", "{a down}")
# handle.map("{l up}", "{a up}")
#
# handle.map("{i down}", "i am here now")
# handle.map("{i up}", " bye")
#
# handle.map("{j down}", "mib")
# handle.map("{j up}", " mab")
#
# counter = 0
#
#
# def hello():
#     handle.send_modifier("{shift down}")
#     handle.send("hi")
#     handle.send_modifier("{shift up}")
#     # global counter
#     # print("counter: {}".format(counter))
#     # counter = counter + 1
#
#
# handle.map("m", hello)


# def hi(cls):
#    print("window is: {}".format(cls))
#
# window = map2.Window()
# foo = window.on_window_change(hi)
# window.remove_on_window_change(foo)
#
# foo = window.on_window_change(hi)
# foo = window.on_window_change(hi)
reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd",
    # "/dev/input/by-id/usb-Logitech_USB_Receiver-if01-event-.*",
    # "/dev/input/by-id/usb-Logitech_USB_Receiver-if02-event-.*",
    # "/dev/input/by-id/usb-Logitech_G700s_Rechargeable_Gaming_Mouse_017DF9570007-.*-event-.*",
])
# writer = map2.Writer(reader)
mapper = map2.Mapper(reader)
mapper.map_key("+a", "b")


def l1(): print("shift down"); reader.send_raw("{shift down}")
mapper.map("{shift down}", l1)
def l2(): print("shift up"); reader.send_raw("{shift up}")
mapper.map("+{shift up}", l2)

mapper2 = map2.Mapper(mapper)
# mapper2.map_key("b", "c")


# def hi():
#     # print("hi", 10 / 0)
#     reader.send("{shift down}")
#
# mapper.map("j", hi)
# mapper2.map("k", hi)



writer = map2.Writer(mapper2, options={
    "capabilities": {
        "keyboard": True
    }
})

#writer2 = map2.Writer(mapper2, options={
#    "capabilities": {
#        "rel": True,
#        "buttons": True,
#    }
#})

# import asyncio
# asyncio.new_event_loop()

async def foo():
    print("start")
    await asyncio.sleep(1)
    print("done")

# writer.map("a", foo)


time.sleep(8)
