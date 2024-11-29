import map2

reader = map2.Reader(patterns=[ "/dev/input/by-id/example"])
mapper = map2.Mapper()
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

map2.link([reader, mapper, writer])

mapper.map("a", "b")

mapper.map("#q", "t")
# mapper.map_key("#q down", "t down")
# mapper.map_key("#q repeat", "t repeat")
# mapper.map_key("#q up", "t up")

# compatible
mapper.map_key("#1 down", "^a down")
mapper.nop("#1 repeat")
mapper.map_key("#1 up", "^a up")

# compatible
mapper.map_key("#2 down", "^b down")
def fn(): pass
mapper.map("#2 repeat", fn)
mapper.map_key("#2 up", "^b up")

# not compatible - repeat has different mods
mapper.map_key("#3 down", "^c down")
mapper.map_key("#3 up", "^c up")

# not compatible - down and up have different mods
mapper.map_key("#4 down", "^d down")
mapper.nop("#4 repeat")
mapper.map_key("#4 up", "!d up")


# mods?
# mapper.map_key("#1 down", "a down", restore_mods=True)
# mapper.map_key("#2", "b")

