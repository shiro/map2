import map2
import time

foo = 33


def hi(a):
    print("got key: '{}' with value '{}'".format(a.code, a.value))


# map2.sum_as_string(5, 20, hi)
handle = map2.setup(hi)

print("handle obtained: {}".format(handle))

handle.map("a","c")
handle.map("+a","b")
handle.map("c","+a")
handle.map("d","{a down}")

time.sleep(5)
