import map2
import time

foo = 33


def hi(a):
    print("got key: '{}' with value '{}'".format(a.code, a.value))


# map2.sum_as_string(5, 20, hi)
handle = map2.setup(hi)

print("handle obtained: {}".format(handle))

handle.map("^f13","a")

time.sleep(5)
