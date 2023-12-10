import map2

reader = map2.Reader(patterns=[ "/dev/input/by-id/example"])
mapper = map2.Mapper()
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

map2.link([reader, mapper, writer])

mapper.map("a", "b")
