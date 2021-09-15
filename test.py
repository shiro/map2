import map2
import time

foo = 33


def hi(a):
    print("got key: '{}' with value '{}'".format(a.code, a.value))


# map2.sum_as_string(5, 20, hi)
handle = map2.setup(hi)

print("handle obtained: {}".format(handle))

handle.map("a", "c")
handle.map("+a", "b")
handle.map("d", "{a down}")

handle.map("{l down}", "a")
handle.map("{l up}", "b")

counter = 0

def hello():
    global counter
    print("counter: {}".format(counter))
    counter = counter + 1


handle.map("{m down}", hello)

time.sleep(5)
