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


def hi(cls):
    print("window is: {}".format(cls))

window = map2.Window()
foo = window.on_window_change(hi)
window.remove_on_window_change(foo)

foo = window.on_window_change(hi)
foo = window.on_window_change(hi)

time.sleep(5)