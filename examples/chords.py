import map2

reader = map2.Reader(filters=[ "/dev/input/by-id/example"])
mapper = map2.ChordMapper()
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

map2.link([reader, mapper, writer])

mapper.map(["a", "b"], "c")
mapper.map(["a", "d"], "e")

counter = 0

def increment(*args):
  global counter
  counter += 1
mapper.map(["c", "d"], increment)
