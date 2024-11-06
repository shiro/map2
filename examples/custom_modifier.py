import map2

reader = map2.Reader(patterns=["/dev/input/by-id/example"])
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

# a modifier mapper transforms any key into a modifier key, we'll use it on capslock here
mapper = map2.ModifierMapper("capslock")

map2.link([reader, mapper, writer])


# map 'capslock'+'a' to 'b'
mapper.map("a", "b")
