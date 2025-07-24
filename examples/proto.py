import map2

reader = map2.Reader(filters=["/dev/input/by-id/example"])
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

m = map2.Mapper()
# mod = map2.Modifier("z")

# map2.link([reader, mod, m, writer])

# print(m + "ok")


# mod = map2.Modifier("a")

# mapper.map(mod+"!b", "c")
# mapper.map(mod.down+"!b", "c")

# mapper.map(mod.key+"!b", "c")

# mapper.map([mod, "!b"], "c")
# mapper.map([mod.down, "!b"], "c")
