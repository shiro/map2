import map2

reader = map2.Reader(patterns=[ "/dev/input/by-id/example"])
mapper = map2.TextMapper()
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

map2.link([reader, mapper, writer])

mapper.map("hello", "bye")

# counter = 0
#
# def increment():
#   global counter
#   counter += 1
# mapper.map(["c", "d"], increment)
